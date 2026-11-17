/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! Filesystem operations with useful errors that are still `io::Error` and
//! preserve the causal chain back to the original `io::Error`.
//!
//! The `io::Error`s returned here have the same `kind` as the original
//! error. So for most practical uses they can be treated the same.
//! The main difference is that `raw_os_error` returns `None`.
//! To get the actual `raw_os_error`, you have to go through `source`.

use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
struct CustomError {
    from: (&'static str, PathBuf),
    to: Option<(&'static str, PathBuf)>,
    source: io::Error,
}

impl StdError for CustomError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (from_action, from_path) = &self.from;
        write!(f, "failed to {} `{}`", from_action, from_path.display())?;
        if let Some((to_action, to_path)) = self.to.as_ref() {
            write!(f, " {} `{}`", to_action, to_path.display())?;
        }
        Ok(())
    }
}

fn wrap1<P: AsRef<Path>>(source: io::Error, action: &'static str, path: P) -> io::Error {
    io::Error::new(
        source.kind(),
        CustomError {
            from: (action, path.as_ref().into()),
            to: None,
            source,
        },
    )
}

fn wrap2<P: AsRef<Path>, Q: AsRef<Path>>(
    source: io::Error,
    from_action: &'static str,
    from_path: P,
    to_action: &'static str,
    to_path: Q,
) -> io::Error {
    io::Error::new(
        source.kind(),
        CustomError {
            from: (from_action, from_path.as_ref().into()),
            to: Some((to_action, to_path.as_ref().into())),
            source,
        },
    )
}

pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    fs::canonicalize(&path).map_err(|source| wrap1(source, "canonicalize", path))
}

#[cfg_attr(not(dotslash_internal), expect(dead_code))]
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    fs::copy(&from, &to).map_err(|source| wrap2(source, "copy from", from, "to", to))
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs::create_dir_all(&path).map_err(|source| wrap1(source, "create directory", path))
}

pub fn file_create<P: AsRef<Path>>(path: P) -> io::Result<fs::File> {
    fs::File::create(&path).map_err(|source| wrap1(source, "create file", path))
}

pub fn file_open<P: AsRef<Path>>(path: P) -> io::Result<fs::File> {
    fs::File::open(&path).map_err(|source| wrap1(source, "open file", path))
}

#[cfg_attr(all(windows, not(dotslash_internal), not(test)), expect(dead_code))]
pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<fs::Metadata> {
    fs::metadata(&path).map_err(|source| wrap1(source, "get metadata for", path))
}

pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> io::Result<fs::Metadata> {
    fs::symlink_metadata(&path).map_err(|source| wrap1(source, "get symlink_metadata for", path))
}

pub fn namedtempfile_new_in<P: AsRef<Path>>(path: P) -> io::Result<tempfile::NamedTempFile> {
    tempfile::NamedTempFile::new_in(&path)
        .map_err(|source| wrap1(source, "create temp file in", path))
}

#[cfg_attr(not(dotslash_internal), expect(dead_code))]
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    fs::read(&path).map_err(|source| wrap1(source, "read file", path))
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<fs::ReadDir> {
    fs::read_dir(&path).map_err(|source| wrap1(source, "read directory", path))
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(&path).map_err(|source| wrap1(source, "read file", path))
}

pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    fs::rename(&from, &to).map_err(|source| wrap2(source, "rename from", from, "to", to))
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs::remove_file(&path).map_err(|source| wrap1(source, "remove file", path))
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs::remove_dir_all(&path).map_err(|source| wrap1(source, "remove dir all", path))
}

pub fn set_permissions<P: AsRef<Path>>(path: P, perm: fs::Permissions) -> io::Result<()> {
    fs::set_permissions(&path, perm).map_err(|source| wrap1(source, "set permissions for", path))
}

pub fn tempdir_in<P: AsRef<Path>>(path: P) -> io::Result<tempfile::TempDir> {
    tempfile::tempdir_in(&path).map_err(|source| wrap1(source, "create temp dir in", path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let err = metadata("path/to/fake/file").unwrap_err();
        assert_eq!(
            format!("{}", &err),
            "failed to get metadata for `path/to/fake/file`",
        );
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(err.raw_os_error(), None);

        let dyn_source = err.source().unwrap();
        assert_eq!(
            format!("{}", &dyn_source),
            if cfg!(windows) {
                "The system cannot find the path specified. (os error 3)"
            } else {
                "No such file or directory (os error 2)"
            },
        );
        let source = dyn_source.downcast_ref::<io::Error>().unwrap();
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
        assert_eq!(
            source.raw_os_error(),
            if cfg!(windows) { Some(3) } else { Some(2) },
        );
    }

    #[test]
    fn test_rename() {
        let err = rename("path/to/fake/file", "fake").unwrap_err();
        assert_eq!(
            format!("{}", &err),
            "failed to rename from `path/to/fake/file` to `fake`",
        );
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(err.raw_os_error(), None);
    }
}
