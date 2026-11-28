/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::config::ArtifactEntry;
use crate::curl::CurlCommand;
use crate::curl::CurlError;
use crate::curl::FetchContext;
use crate::provider::Provider;
use crate::util::FileLock;
use crate::util::HttpStatus;

const AUTH_HINT: &str = "the server rejected the request as unauthorized; to authenticate, add a \
     `machine <host> login <user> password <token>` entry to your ~/.netrc file \
     (%USERPROFILE%\\_netrc on Windows)";

pub struct HttpProvider {}

#[derive(Deserialize, Debug)]
struct HttpProviderConfig {
    url: String,
}

impl Provider for HttpProvider {
    fn fetch_artifact(
        &self,
        provider_config: &Value,
        destination: &Path,
        _fetch_lock: &FileLock,
        artifact_entry: &ArtifactEntry,
    ) -> anyhow::Result<()> {
        let HttpProviderConfig { url } = <_>::deserialize(provider_config)?;
        let curl_cmd = CurlCommand::new(url.as_ref());
        // Currently, we always disable the progress bar, but we plan to add a
        // configuration option to enable it.
        let show_progress = false;
        let fetch_context = FetchContext {
            artifact_name: url.as_str(),
            content_length: artifact_entry.size,
            show_progress,
        };
        curl_cmd
            .get_request(destination, &fetch_context)
            .map_err(|e| {
                let is_auth_error = matches!(
                    e,
                    CurlError::HttpStatus(_, HttpStatus::Unauthorized | HttpStatus::Forbidden)
                );
                let err = anyhow::Error::new(e);
                let err = if is_auth_error {
                    err.context(AUTH_HINT)
                } else {
                    err
                };
                err.context(format!("failed to fetch `{}`", url))
            })?;
        Ok(())
    }
}
