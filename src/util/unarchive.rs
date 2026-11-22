/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Seek;
use std::path::Path;

use bzip2::read::BzDecoder;
use flate2::bufread::MultiGzDecoder;
#[cfg(not(dotslash_internal))]
use liblzma::bufread::XzDecoder;
use tar::Archive;
#[cfg(not(dotslash_internal))]
use zip::ZipArchive;
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::util::fs_ctx;

#[derive(Copy, Clone)]
pub enum ArchiveType {
    Tar,
    #[cfg(not(dotslash_internal))]
    Bzip2,
    TarBzip2,
    #[cfg(not(dotslash_internal))]
    Gz,
    TarGz,
    #[cfg(not(dotslash_internal))]
    Xz,
    #[cfg(not(dotslash_internal))]
    TarXz,
    #[cfg(not(dotslash_internal))]
    Zstd,
    TarZstd,
    #[cfg(not(dotslash_internal))]
    Zip,
}

/// Attempts to extract the tar/zip archive into the specified directory
/// or file.
///
/// To extract tars, this uses the tar crate (https://crates.io/crates/tar)
/// directly. Those who create compressed artifacts for DotSlash are
/// responsible for ensuring they can be decompressed with its version of tar.
pub fn unarchive<R>(reader: R, destination: &Path, archive_type: ArchiveType) -> io::Result<()>
where
    R: BufRead + Seek,
{
    #[cfg(not(dotslash_internal))]
    if matches!(archive_type, ArchiveType::Zip) {
        let destination = fs_ctx::canonicalize(destination)?;
        let mut archive = ZipArchive::new(reader)?;
        archive.extract(destination)?;
        return Ok(());
    }

    unarchive_stream(reader, destination, archive_type)
}

/// Like [`unarchive`], but for a reader that cannot seek (a pipe, a socket).
///
/// Every archive type except zip is a forward-only stream, so this lets the
/// extraction overlap with the download that produces `reader`.
pub fn unarchive_stream<R>(
    reader: R,
    destination: &Path,
    archive_type: ArchiveType,
) -> io::Result<()>
where
    R: BufRead,
{
    match archive_type {
        ArchiveType::Tar => unpack_tar(reader, destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Bzip2 => write_out(BzDecoder::new(reader), destination),
        ArchiveType::TarBzip2 => unpack_tar(BzDecoder::new(reader), destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Gz => write_out(MultiGzDecoder::new(reader), destination),
        ArchiveType::TarGz => unpack_tar(MultiGzDecoder::new(reader), destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Xz => write_out(XzDecoder::new(reader), destination),
        #[cfg(not(dotslash_internal))]
        ArchiveType::TarXz => unpack_tar(XzDecoder::new(reader), destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Zstd => write_out(ZstdDecoder::with_buffer(reader)?, destination),
        ArchiveType::TarZstd => unpack_tar(ZstdDecoder::with_buffer(reader)?, destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Zip => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "zip archives need a seekable reader",
        )),
    }
}

#[cfg(not(dotslash_internal))]
fn write_out<R>(mut reader: R, destination_dir: &Path) -> io::Result<()>
where
    R: Read,
{
    let mut output_file = fs_ctx::file_create(destination_dir)?;
    io::copy(&mut reader, &mut output_file)?;
    Ok(())
}

fn unpack_tar<R>(reader: R, destination_dir: &Path) -> io::Result<()>
where
    R: Read,
{
    // The destination dir is canonicalized for the benefit of Windows, but we
    // do it on all platforms for consistency of behavior.
    //
    // Windows has a path length limit of 255 chars. "Extended-length paths"[1]
    // are paths starting with `\\?\`. These are not subject to the length
    // limit, but have other issues: they cannot use forward slashes.
    //
    // `fs::canonicalize` will both prefix the path with `\\?\` and normalize
    // the slashes[2]. This is important because we don't know the depth of the
    // tarball file structure (so we need to avoid possible path length
    // limits), and we don't know if the destination path is mixing slashes.
    //
    // We only use extended-length paths here and not earlier because you
    // can't exec `.bat` files with `\\?\` (although `.exe` files are ok).
    //
    // We canonicalize for all platforms because `fs::canonicalize` can
    // error[3] and not everyone can test on Windows.
    //
    // [1] https://docs.microsoft.com/en-us/windows/desktop/FileIO/naming-a-file#maxpath
    // [2] https://doc.rust-lang.org/std/fs/fn.canonicalize.html#platform-specific-behavior
    // [3] https://doc.rust-lang.org/std/fs/fn.canonicalize.html#errors

    let destination_dir = fs_ctx::canonicalize(destination_dir)?;

    let mut archive = Archive::new(reader);
    archive.set_preserve_permissions(true);
    archive.set_preserve_mtime(true);
    archive.unpack(destination_dir)
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use flate2::Compression;
    use flate2::write::GzEncoder;

    use super::*;

    fn gzip_members(parts: &[&[u8]]) -> io::Result<Vec<u8>> {
        let mut compressed = Vec::new();
        for part in parts {
            let mut encoder = GzEncoder::new(&mut compressed, Compression::default());
            encoder.write_all(part)?;
            encoder.finish()?;
        }
        Ok(compressed)
    }

    #[test]
    #[cfg(not(dotslash_internal))]
    fn gzip_multiple_members() -> io::Result<()> {
        let mut compressed = gzip_members(&[b"hello", b" world"])?;
        let dir = tempfile::tempdir()?;
        let destination = dir.path().join("output");

        unarchive(io::Cursor::new(&compressed), &destination, ArchiveType::Gz)?;

        assert_eq!(std::fs::read(&destination)?, b"hello world");

        let crc_offset = compressed.len() - 8;
        compressed[crc_offset] ^= 1;
        assert!(unarchive(io::Cursor::new(compressed), &destination, ArchiveType::Gz).is_err());
        Ok(())
    }

    #[test]
    fn tar_gzip_multiple_members() -> io::Result<()> {
        let content = b"hello world";
        let mut archive = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        archive.append_data(&mut header, "tool", &content[..])?;
        let archive = archive.into_inner()?;
        let compressed = gzip_members(&[&archive[..512], &archive[512..]])?;
        let dir = tempfile::tempdir()?;

        unarchive(io::Cursor::new(compressed), dir.path(), ArchiveType::TarGz)?;

        assert_eq!(std::fs::read(dir.path().join("tool"))?, content);
        Ok(())
    }
}
