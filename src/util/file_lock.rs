/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! Wrapper around `std::fs::File::lock`.

use std::fs::File;
use std::fs::TryLockError;
use std::io;
#[cfg(unix)]
use std::os::fd::AsFd;
#[cfg(unix)]
use std::os::fd::BorrowedFd;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileLockError {
    #[error("failed to create lock file `{0}`")]
    Create(PathBuf, #[source] io::Error),

    #[error("failed to get exclusive lock `{0}`")]
    LockExclusive(PathBuf, #[source] io::Error),

    #[cfg_attr(
        not(dotslash_internal),
        expect(
            dead_code,
            reason = "only constructed by `FileLock::acquire_shared_lock`"
        )
    )]
    #[error("failed to get shared lock `{0}`")]
    LockShared(PathBuf, #[source] io::Error),

    #[cfg_attr(
        not(dotslash_internal),
        expect(dead_code, reason = "only constructed by `FileLock::try_acquire`")
    )]
    #[error("`{0}` is held by another process")]
    WouldBlock(PathBuf),
}

#[derive(Debug, Default)]
pub struct FileLock {
    /// If file is Some, then it is holding the lock.
    file: Option<File>,
}

impl FileLock {
    pub fn acquire<P>(path: P) -> Result<FileLock, FileLockError>
    where
        P: AsRef<Path>,
    {
        fn inner(path: &Path) -> Result<FileLock, FileLockError> {
            let lock_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .map_err(|e| FileLockError::Create(path.to_path_buf(), e))?;

            lock_file
                .lock()
                .map_err(|e| FileLockError::LockExclusive(path.to_path_buf(), e))?;

            Ok(FileLock {
                file: Some(lock_file),
            })
        }
        inner(path.as_ref())
    }

    /// Like [`FileLock::acquire`], but reports [`FileLockError::WouldBlock`]
    /// instead of waiting when another process holds the lock. The garbage
    /// collector uses this so that one long-lived artifact cannot stall the
    /// whole sweep.
    #[cfg_attr(
        not(dotslash_internal),
        expect(dead_code, reason = "only used by the internal release")
    )]
    pub fn try_acquire<P>(path: P) -> Result<FileLock, FileLockError>
    where
        P: AsRef<Path>,
    {
        fn inner(path: &Path) -> Result<FileLock, FileLockError> {
            let lock_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .map_err(|e| FileLockError::Create(path.to_path_buf(), e))?;

            match lock_file.try_lock() {
                Ok(()) => Ok(FileLock {
                    file: Some(lock_file),
                }),
                Err(TryLockError::WouldBlock) => Err(FileLockError::WouldBlock(path.to_path_buf())),
                Err(TryLockError::Error(e)) => {
                    Err(FileLockError::LockExclusive(path.to_path_buf(), e))
                }
            }
        }
        inner(path.as_ref())
    }

    #[cfg_attr(
        not(dotslash_internal),
        expect(dead_code, reason = "only used by the internal release")
    )]
    pub fn acquire_shared_lock<P>(path: P) -> Result<FileLock, FileLockError>
    where
        P: AsRef<Path>,
    {
        fn inner(path: &Path) -> Result<FileLock, FileLockError> {
            let lock_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .map_err(|e| FileLockError::Create(path.to_path_buf(), e))?;

            lock_file
                .lock_shared()
                .map_err(|e| FileLockError::LockShared(path.to_path_buf(), e))?;

            Ok(FileLock {
                file: Some(lock_file),
            })
        }
        inner(path.as_ref())
    }

    /// The descriptor holding the lock, if one is held.
    #[cfg(unix)]
    #[cfg_attr(
        not(dotslash_internal),
        expect(dead_code, reason = "only used by the internal release")
    )]
    pub fn as_fd(&self) -> Option<BorrowedFd<'_>> {
        self.file.as_ref().map(AsFd::as_fd)
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            drop(file.unlock());
        }
    }
}
