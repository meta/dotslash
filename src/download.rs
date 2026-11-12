/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use rand::distr::Distribution;
use serde_json::Value;
use sha2::Digest as _;
use sha2::Sha256;

use crate::artifact_location::ArtifactLocation;
use crate::config::ArtifactEntry;
use crate::config::HashAlgorithm;
use crate::config::ProvidersOrder;
use crate::digest::Digest;
use crate::fetch_method::ArtifactFormat;
use crate::provider::ProviderFactory;
use crate::util;
use crate::util::FileLock;
use crate::util::FileLockError;
use crate::util::fs_ctx;
use crate::util::unarchive;

pub const DEFAULT_PROVDIER_TYPE: &str = "http";

/// This function is responsible for:
/// 1. Downloading an artifact into a temp location within $DOTSLASH_CACHE.
/// 2. Verifying that the size and digest match the ArtifactEntry.
/// 3. Decompressing the artifact, as appropriate.
/// 4. Atomically moving it from its temp location to its final location.
pub fn download_artifact<P: ProviderFactory>(
    artifact_entry: &ArtifactEntry,
    artifact_location: &ArtifactLocation,
    provider_factory: &P,
) -> anyhow::Result<()> {
    let artifact_parent_dir = artifact_location
        .artifact_directory
        .parent()
        .with_context(|| {
            format!(
                "no parent for artifact_directory `{}`",
                artifact_location.artifact_directory.display()
            )
        })?;

    fs_ctx::create_dir_all(artifact_parent_dir)?;

    // We must maintain a reference to the FileLock until the download is complete.
    let file_lock = acquire_download_lock_for_artifact(artifact_location)
        .context("failed to get artifact lock")?;

    // Another process may have fetched the artifact while we were waiting for
    // the lock.
    if artifact_location.executable.exists() {
        return Ok(());
    }

    // Record warnings: only reported if no provider succeeds.
    let mut warnings = vec![];

    // Build a list of provider references,
    // and if randomization is enabled, shuffle them.
    let mut rng = rand::rng();
    let providers = providers_in_order(
        &mut rng,
        &artifact_entry.providers,
        artifact_entry.providers_order,
    )?;

    for provider_config in providers {
        // This must be a sibling to the final artifact_location so that we can
        // atomically move it into place.
        let temp_dir_to_mv = fs_ctx::tempdir_in(artifact_parent_dir)?;
        let fetch_destination: PathBuf = {
            let fetch_destination = fs_ctx::namedtempfile_new_in(artifact_parent_dir)
                .context("failed to create fetch temp path")?
                .into_temp_path();
            // fetch_destination is dropped after this and is removed from
            // disk. This is deliberate since we want a unique path and not
            // necessarily the file.
            fetch_destination.to_path_buf()
        };

        let provider_type = get_provider_type(provider_config)?;
        let provider = provider_factory.get_provider(provider_type)?;
        match provider.fetch_artifact(
            provider_config,
            &fetch_destination,
            &file_lock,
            artifact_entry,
        ) {
            Ok(()) => match verify_artifact(&fetch_destination, artifact_entry) {
                Ok(()) => {
                    unpack_verified_artifact(
                        &fetch_destination,
                        temp_dir_to_mv.path(),
                        artifact_entry.format,
                        artifact_entry.path.as_str(),
                    )?;
                    if artifact_entry.readonly {
                        util::make_tree_entries_read_only(temp_dir_to_mv.path())?;
                    }
                    util::mv_no_clobber(&temp_dir_to_mv, &artifact_location.artifact_directory)?;
                    if artifact_entry.readonly {
                        // Note the following appears to work on Linux but not
                        // macOS:
                        //
                        // ```
                        // /tmp$ mkdir foo
                        // /tmp$ chmod -w foo
                        // /tmp$ mv foo bar
                        // ```
                        //
                        // so we have to do the final `chmod -w` after the `mv`.
                        // While we could also do the full `chmod -R -w` after
                        // the `mv`, that is a bit riskier because a
                        // simultaneous invocation of the DotSlash file would be
                        // able to use the artifact before `chmod -R -w`
                        // finishes.
                        let metadata =
                            fs_ctx::symlink_metadata(&artifact_location.artifact_directory)?;
                        let mut perms = metadata.permissions();
                        perms.set_readonly(true);
                        fs_ctx::set_permissions(&artifact_location.artifact_directory, perms)?;
                    }
                    return Ok(());
                }
                Err(e) => warnings.push(format!("warning: failed to verify artifact {:?}", e)),
            },
            Err(e) => warnings.push(format!("failed to fetch artifact: {:?}", e)),
        }
    }

    Err(anyhow::format_err!(
        "no providers succeeded. warnings:\n{}",
        warnings.join("\n")
    ))
}

fn providers_in_order<'b>(
    rng: &mut impl rand::Rng,
    providers: &'b [Value],
    providers_order: ProvidersOrder,
) -> anyhow::Result<Vec<&'b Value>> {
    let mut ordered_providers: Vec<&Value> = Vec::with_capacity(providers.len());
    match providers_order {
        ProvidersOrder::Sequential => {
            // Just use the order in the config.
            for provider_config in providers {
                ordered_providers.push(provider_config);
            }
        }

        ProvidersOrder::WeightedRandom => {
            // Assign weights to each provider based on the "weight" field.
            // Defaults to 1 if not specified.
            let mut weights: Vec<u64> = Vec::with_capacity(providers.len());
            for (idx, provider_config) in providers.iter().enumerate() {
                let weight_value = provider_config.get("weight");

                let weight = match weight_value {
                    None => 1u64,
                    Some(weight) => weight.as_u64().with_context(|| {
                        format!("provider[{}]: weight must be a non-negative integer", idx)
                    })?,
                };

                if weight == 0 {
                    return Err(anyhow::anyhow!(
                        "provider[{}]: weight must be greater than 0",
                        idx
                    ));
                }

                weights.push(weight);
            }

            // Shuffle the providers using weighted sampling of indexes
            // without duplicates in the result.
            let dist = rand::distr::weighted::WeightedIndex::new(&weights)
                .map_err(|e| anyhow::anyhow!("error initializing weights: {}", e))?;
            let mut seen = std::collections::HashSet::new();
            while ordered_providers.len() < providers.len() {
                let idx = dist.sample(rng);
                if seen.insert(idx) {
                    ordered_providers.push(&providers[idx]);
                }
            }
        }
    }

    Ok(ordered_providers)
}

fn get_provider_type(provider_config: &Value) -> anyhow::Result<&str> {
    match provider_config.get("type") {
        Some(v) => v.as_str().context("type must map to a string"),
        None => Ok(DEFAULT_PROVDIER_TYPE),
    }
}

fn verify_artifact(
    artifact_temp_location: &Path,
    artifact_entry: &ArtifactEntry,
) -> anyhow::Result<()> {
    // First, verify the hash and digest.
    let mut file = File::options()
        .read(true)
        .open(artifact_temp_location)
        .with_context(|| {
            format!(
                "failed to open fetched artifact `{}`",
                artifact_temp_location.display()
            )
        })?;
    let (size_in_bytes, digest) = match artifact_entry.hash {
        HashAlgorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            io::copy(&mut file, &mut hasher).map(|size_in_bytes| {
                let digest = format!("{}", hasher.finalize().to_hex());
                (size_in_bytes, digest)
            })
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            io::copy(&mut file, &mut hasher).map(|size_in_bytes| {
                let digest = format!("{:x}", hasher.finalize());
                (size_in_bytes, digest)
            })
        }
    }
    .with_context(|| {
        format!(
            "failed to calculate digest for fetched archive `{}`",
            artifact_temp_location.display()
        )
    })?;
    drop(file);

    if size_in_bytes != artifact_entry.size {
        return Err(anyhow::format_err!(
            "fetched artifact `{}` has incorrect size: {} bytes vs expected {} bytes",
            artifact_temp_location.display(),
            size_in_bytes,
            artifact_entry.size
        ));
    }

    let digest = Digest::try_from(digest)?;
    if digest != artifact_entry.digest {
        return Err(anyhow::format_err!(
            "fetched artifact `{}` has incorrect digest: {} vs expected {}",
            artifact_temp_location.display(),
            digest,
            artifact_entry.digest
        ));
    }

    Ok(())
}

/// Unpacks the verified artifact. When this function exits, the contents of
/// `temp_dir_to_mv` should be ready to be moved into the final location.
fn unpack_verified_artifact(
    fetched_artifact: &Path,
    temp_dir_to_mv: &Path,
    format: ArtifactFormat,
    artifact_entry_path: &str,
) -> anyhow::Result<()> {
    // Container artifacts get unarchived into directories.
    // Non-container artifacts get written directly to a file.
    let final_artifact_path = if format.is_container() {
        Cow::Borrowed(temp_dir_to_mv)
    } else {
        let final_artifact_path = temp_dir_to_mv.join(artifact_entry_path);
        let parent = final_artifact_path.parent().unwrap();
        if parent != Path::new("") {
            fs_ctx::create_dir_all(parent)?;
        }
        Cow::Owned(final_artifact_path)
    };

    if let Some(archive_type) = format.as_archive_type() {
        let reader = BufReader::new(fs_ctx::file_open(fetched_artifact)?);
        unarchive::unarchive(reader, &final_artifact_path, archive_type)?;
    } else {
        fs_ctx::rename(fetched_artifact, &final_artifact_path)?;
    }

    if !format.is_container() {
        #[cfg(unix)]
        util::chmodx(final_artifact_path).context("failed to make path executable")?;
    }

    Ok(())
}

/// Attempts to acquire an advisory lock for a lock file in the DotSlash cache
/// that corresponds to the artifact specified by `scheme`, creating the file
/// if necessary. This should be done before download_artifact() is called in
/// order to help prevent concurrent fetches.
///
/// Note that this system is not 100% foolproof: it serves as a "best effort" to
/// avoid concurrent fetches, but it is not guaranteed to prevent them.
///
/// Fortunately, download_artifact() is designed to be resilient in the face of
/// concurrent fetches, so locking is only a performance optimization, not a
/// required safeguard.
pub fn acquire_download_lock_for_artifact(
    artifact_location: &ArtifactLocation,
) -> anyhow::Result<FileLock> {
    if let Some(lock_dir) = artifact_location.lock_path.parent() {
        if fs_ctx::create_dir_all(lock_dir).is_ok() {
            match FileLock::acquire(&artifact_location.lock_path) {
                Ok(file_lock) => return Ok(file_lock),
                Err(err @ FileLockError::LockExclusive(..)) => return Err(err.into()),
                Err(FileLockError::Create(..) | FileLockError::LockShared(..)) => {}
            }
        }
    }
    Ok(FileLock::default())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rand::SeedableRng;

    use super::*;
    use crate::config::Arg0;
    use crate::provider::Provider;

    struct PanickingProviderFactory;

    impl ProviderFactory for PanickingProviderFactory {
        fn get_provider(&self, provider_type: &str) -> anyhow::Result<Box<dyn Provider>> {
            panic!("unexpected fetch with provider type `{provider_type}`");
        }
    }

    /// Simulates `download_artifact()` being called when the executable is
    /// already in the cache. Callers only call it when they didn't find the
    /// executable, but it can be there anyway by the time
    /// `download_artifact()` holds the lock:
    ///
    /// - Another process fetched the artifact after the caller checked,
    ///   usually while this process waited for the lock. That process may
    ///   have run the same DotSlash file, or a different one whose artifact
    ///   is the same archive (e.g. `node` and `npx` from one Node.js tarball).
    /// - Running the executable failed with a not-found error even though it
    ///   exists, e.g. because its shebang interpreter is missing.
    ///
    /// In either case, `download_artifact()` should not download the artifact
    /// again.
    #[test]
    fn download_artifact_skips_fetch_if_already_present() {
        let temp_dir = tempfile::tempdir().unwrap();
        let artifact_directory = temp_dir.path().join("artifact");
        let executable = artifact_directory.join("hello");
        fs::create_dir(&artifact_directory).unwrap();
        fs::write(&executable, "").unwrap();
        let artifact_location = ArtifactLocation {
            artifact_directory,
            executable,
            lock_path: temp_dir.path().join("lock"),
            arg0: Arg0::UnderlyingExecutable,
        };
        let artifact_entry = ArtifactEntry {
            size: 0,
            hash: HashAlgorithm::Sha256,
            digest: Digest::try_from(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_owned(),
            )
            .unwrap(),
            format: ArtifactFormat::Plain,
            path: "hello".parse().unwrap(),
            providers: vec![
                serde_json::from_str(r#"{"url": "https://example.com/hello"}"#).unwrap(),
            ],
            arg0: Arg0::UnderlyingExecutable,
            readonly: true,
            providers_order: ProvidersOrder::Sequential,
        };

        download_artifact(
            &artifact_entry,
            &artifact_location,
            &PanickingProviderFactory,
        )
        .expect("download_artifact should succeed without fetching");
    }

    #[test]
    fn providers_in_order_sequential() {
        let mut rng = rand::rng(); // doesn't matter

        let providers = vec![
            serde_json::from_str(r#"{"type": "a"}"#).unwrap(),
            serde_json::from_str(r#"{"type": "b"}"#).unwrap(),
            serde_json::from_str(r#"{"type": "c"}"#).unwrap(),
        ];

        let ordered_providers =
            providers_in_order(&mut rng, &providers, ProvidersOrder::Sequential)
                .expect("failed to order providers");

        assert_eq!(
            ordered_providers,
            vec![&providers[0], &providers[1], &providers[2]]
        );
    }

    #[test]
    fn providers_in_order_weighted_random_default_weights() {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42); // deterministic for testing

        let providers = vec![
            serde_json::from_str(r#"{"type": "a"}"#).unwrap(),
            serde_json::from_str(r#"{"type": "b"}"#).unwrap(),
            serde_json::from_str(r#"{"type": "c"}"#).unwrap(),
        ];

        let ordered_providers =
            providers_in_order(&mut rng, &providers, ProvidersOrder::WeightedRandom)
                .expect("failed to order providers");

        assert_eq!(
            ordered_providers,
            vec![&providers[2], &providers[0], &providers[1]]
        );
    }

    #[test]
    fn providers_in_order_weighted_random_custom_weights() {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42); // deterministic for testing

        let providers = vec![
            serde_json::from_str(r#"{"type": "a", "weight": 1}"#).unwrap(),
            serde_json::from_str(r#"{"type": "b", "weight": 2}"#).unwrap(),
            serde_json::from_str(r#"{"type": "c", "weight": 10}"#).unwrap(),
            serde_json::from_str(r#"{"type": "d", "weight": 2}"#).unwrap(),
        ];

        let ordered_providers =
            providers_in_order(&mut rng, &providers, ProvidersOrder::WeightedRandom)
                .expect("failed to order providers");

        assert_eq!(
            ordered_providers,
            vec![&providers[2], &providers[3], &providers[1], &providers[0]]
        );
    }

    #[test]
    fn providres_in_order_weighted_random_zero_weight() {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42); // deterministic for testing

        let providers = vec![
            serde_json::from_str(r#"{"type": "a", "weight": 0}"#).unwrap(),
            serde_json::from_str(r#"{"type": "b", "weight": 2}"#).unwrap(),
        ];

        let result = providers_in_order(&mut rng, &providers, ProvidersOrder::WeightedRandom);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("weight must be greater than 0")
        );
    }

    #[test]
    fn providers_in_order_weighted_random_negative_weight() {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42); // deterministic for testing

        let providers = vec![
            serde_json::from_str(r#"{"type": "a", "weight": -1}"#).unwrap(),
            serde_json::from_str(r#"{"type": "b", "weight": 2}"#).unwrap(),
        ];

        let result = providers_in_order(&mut rng, &providers, ProvidersOrder::WeightedRandom);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("weight must be a non-negative integer")
        );
    }
}
