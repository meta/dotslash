/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

#![allow(non_snake_case)]

mod common;

use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::str;

use tempfile::NamedTempFile;

use crate::common::DotslashTestEnv;
use crate::common::ci;
use crate::common::if_win_else;

#[cfg(unix)]
fn non_utf8_osstring() -> OsString {
    std::os::unix::ffi::OsStringExt::from_vec(b"a/\xc3/c".to_vec())
}

#[cfg(windows)]
fn non_utf8_osstring() -> OsString {
    // Anything between 0xD800–0xDBFF will end up producing a UTF-16
    // character that can't be converted to UTF-8.
    // https://en.wikipedia.org/wiki/UTF-16#Code_points_from_U+010000_to_U+10FFFF
    const UTF16_SURROGATE_RANGE_START: u16 = 0xD800;
    std::os::windows::ffi::OsStringExt::from_wide(&[
        u16::from(b'a'),
        u16::from(b'/'),
        UTF16_SURROGATE_RANGE_START,
        u16::from(b'/'),
        u16::from(b'c'),
    ])
}

#[test]
fn test_non_utf8_osstring() {
    assert_eq!(non_utf8_osstring().to_string_lossy(), "a/\u{FFFD}/c");
}

//
// Providers
//

#[test]
fn http__gz__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_GZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__gz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__gz__print_argv
",
        ));
}

#[test]
fn http__xz__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_XZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__xz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__xz__print_argv
",
        ));
}

#[test]
fn http__zst__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_ZST_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__zst__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__zst__print_argv
",
        ));
}

#[test]
fn http__zip__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_ZIP_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__zip__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__zip__print_argv
",
        ));
}

#[test]
fn http__plain__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__plain__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__plain__print_argv
",
        ));
}

#[test]
fn http__tar_gz__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_gz__print_argv
",
        ));
}

#[test]
fn http__tar_xz__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_TAR_XZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__tar_xz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_xz__print_argv
",
        ));
}

#[test]
fn http__tar_zst__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__tar_zst__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_zst__print_argv
",
        ));
}

// TODO(jonjanzen): This test is broken on GitHub Actions specifically for Intel Macs.
// As of 6/25/2025 we think this is a problem with the GHA runner and we don't really
// care much about intel macOS anymore anyway, so disable:
#[cfg(not(all(target_arch = "x86_64", target_os = "macos")))]
#[test]
fn http__nonexistent_url() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction("[DOTSLASH_FILE]", "[CURRENT_DIR]/tests/fixtures/http__nonexistent_url")
        .path_redaction("[ARTIFACT_LOCATION]", "[DOTSLASH_CACHE_DIR]/[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]")
        .path_redaction("[OUTPUT_FILE]", "[DOTSLASH_CACHE_DIR]/cf/.tmp")
        .redaction(
            "[PROVIDER_URL]",
            "https://github.com/zertosh/dotslash_fixtures/raw/5adea95f2eac6509cad9ca87eb770596a1a21379/fake.tar.gz",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__nonexistent_url")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            format!("\
dotslash error: problem with `[DOTSLASH_FILE]`
caused by: failed to download artifact into cache `[DOTSLASH_CACHE_DIR]` artifact location `[ARTIFACT_LOCATION]`
caused by: no providers succeeded. warnings:
failed to fetch artifact: failed to fetch `[PROVIDER_URL]`

Caused by:
    0: `curl --location --retry 3 --fail --silent --show-error {}--netrc-optional --user-agent [DOTSLASH_USER_AGENT] [PROVIDER_URL] --output [OUTPUT_FILE][..]`
    1: 404 Not Found
", if cfg!(windows) { "--ssl-revoke-best-effort "} else {""}),
        );
}

#[test]
fn http__arg0() -> anyhow::Result<()> {
    let mut test_env = DotslashTestEnv::try_new()?;

    let dotslash_file = "tests/fixtures/http__tar_zst__print_argv";

    let contents = fs::read_to_string(test_env.current_dir().join(dotslash_file))?;

    let tempfile = NamedTempFile::new()?;
    fs::write(
        tempfile.path(),
        contents.replace(
            r#""format": "tar.zst","#,
            r#""format": "tar.zst", "arg0": "underlying-executable","#,
        ),
    )?;

    test_env.path_redaction(
        "[ARTIFACT_EXE]",
        "[DOTSLASH_CACHE_DIR]/[PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
    );

    // Default behavior is "dotslash-file".
    test_env
        .dotslash_command()
        .arg(dotslash_file)
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_zst__print_argv
",
        ));

    // Modified "underlying-executable" behavior.
    test_env
        .dotslash_command()
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
        );

    Ok(())
}

//
// Commands
//

#[test]
fn command_missing() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "dotslash error: no subcommand passed to '--'

See `dotslash --help` for more information.
",
        );
}

#[test]
fn command_no_match() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("fake")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: unknown subcommand passed to '--': `fake`

See `dotslash --help` for more information.
",
        );
}

#[test]
fn dotslash_file_arg_missing() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq("dotslash error: must specify the path to a DotSlash file\n");
}

#[test]
fn dotslash_file_not_found() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("path/to/fake/file")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `path/to/fake/file`
caused by: failed to read DotSlash file
caused by: [IO_ERROR_NOT_FOUND]
",
        );
}

#[test]
fn internal_dotslash_file_detected() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[DOTSLASH_FILE]",
            "[CURRENT_DIR]/tests/fixtures/internal_oncall_file",
        )
        .dotslash_command()
        // Clear PATH so no fallback dotslash binary can be found.
        .env("PATH", "")
        .arg("tests/fixtures/internal_oncall_file")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `[DOTSLASH_FILE]`
caused by: failed to parse DotSlash file
caused by: this appears to be an internal (Meta) DotSlash file (it has an \"oncall\" field) and is not supported by the open-source DotSlash binary
caused by: [..]
",
        );
}

#[test]
fn dotslash_file_is_a_directory() {
    DotslashTestEnv::try_new()
        .unwrap()
        .redaction(
            "[IO_ERROR_IS_A_DIRECTORY]",
            if_win_else!(
                "Access is denied. (os error 5)",
                "Is a directory (os error 21)",
            ),
        )
        .path_redaction("[DOTSLASH_FILE]", "[CURRENT_DIR]/tests/fixtures")
        .dotslash_command()
        .arg("tests/fixtures")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `[DOTSLASH_FILE]`
caused by: failed to read DotSlash file
caused by: [IO_ERROR_IS_A_DIRECTORY]
",
        );
}

#[test]
fn dotslash_file_name_is_non_utf8() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg(non_utf8_osstring())
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `a/\u{FFFD}/c`
caused by: failed to read DotSlash file
caused by: [IO_ERROR_NOT_FOUND]
",
        );
}

//
// "help" Command
//

const HELP_STDERR: &str = "\
usage: dotslash DOTSLASH_FILE [OPTIONS]

DOTSLASH_FILE must be a file that starts with `#!/usr/bin/env dotslash`
and contains a JSON body tells DotSlash how to fetch and run the executable
that DOTSLASH_FILE represents.

All OPTIONS will be forwarded directly to the executable identified by
DOTSLASH_FILE.

Supported platform: [..]

Your DotSlash cache is: [DOTSLASH_CACHE_DIR]

dotslash also has these special experimental commands:
  dotslash --help                   Print this message
  dotslash --version                Print the version of dotslash
  dotslash -- b3sum FILE            Compute blake3 hash
  dotslash -- clean                 Clean dotslash cache
  dotslash -- create-url-entry URL  Generate \"http\" provider entry
  dotslash -- cache-dir             Print path to the cache directory
  dotslash -- download-only [--platform PLATFORM] DOTSLASH_FILE
                                    Download and decompress the artifact
                                    without executing it
                                    PLATFORM: e.g. linux-aarch64 (default: host)
  dotslash -- fetch DOTSLASH_FILE   Prepare for execution, but print exe path
                                    instead of executing
  dotslash -- get-extracted-cache-path DOTSLASH_FILE
                                    Print where the artifact would be cached,
                                    without fetching it
  dotslash -- parse DOTSLASH_FILE   Parse the dotslash file
  dotslash -- sha256 FILE           Compute sha256 sum of the file

Learn more at https://dotslash-cli.com
";

#[test]
fn help_command_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("help")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq(HELP_STDERR);
}

#[test]
fn help_flag_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--help")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq(HELP_STDERR);
}

#[test]
fn help_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("help")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'help' command failed
caused by: expected no arguments but received some
",
        );
}

#[test]
fn help_flag_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--help")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'help' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "version" Command
//

const VERSION_STDOUT: &str = concat!("DotSlash ", env!("CARGO_PKG_VERSION"), "\n");

#[test]
fn version_command_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("version")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(VERSION_STDOUT);
}

#[test]
fn version_flag_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--version")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(VERSION_STDOUT);
}

#[test]
fn version_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("version")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'version' command failed
caused by: expected no arguments but received some
",
        );
}

#[test]
fn version_flag_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--version")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'version' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "b3sum" Command
//

#[test]
fn b3sum_command_ok() -> anyhow::Result<()> {
    let tempfile = NamedTempFile::new()?;
    fs::write(tempfile.path(), "DotSlash Rulez!\n")?;

    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("b3sum")
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("824ecff042b9ac68a33ea5ee027379a09f3da1d5a47f29fc52072809a204db87\n");

    Ok(())
}

//
// "cache-dir" Command
//

#[test]
fn cache_dir_command_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("cache-dir")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("[DOTSLASH_CACHE_DIR]\n");
}

#[test]
fn cache_dir_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("cache-dir")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'cache-dir' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "clean" Command
//

#[test]
fn clean_command_ok() -> anyhow::Result<()> {
    let test_env = DotslashTestEnv::try_new()?;

    let cache_dir = test_env.dotslash_cache();

    // Cache dir exists, but should be empty.
    assert!(cache_dir.exists());
    let is_empty = fs::read_dir(cache_dir)?.next().is_none();
    assert!(is_empty);

    test_env
        .dotslash_command()
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .success();

    assert!(cache_dir.exists());

    test_env
        .dotslash_command()
        .arg("--")
        .arg("clean")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq("Cleaning `[DOTSLASH_CACHE_DIR]`\n");

    assert!(!cache_dir.exists());

    Ok(())
}

#[test]
fn clean_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("clean")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'clean' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "create-url-entry" Command
//

#[test]
fn create_url_entry_tar_gz() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("create-url-entry")
        .arg("https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.gz")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(
            r#"{
  "size": 48689,
  "hash": "blake3",
  "digest": "068464830bd5c276e085a4eab5ef9cc57159f94273db296d6a638e49b78ca55f",
  "format": "tar.gz",
  "path": "TODO: specify the appropriate `path` for this artifact",
  "providers": [
    {
      "url": "https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.gz"
    }
  ]
}
"#,
        );
}

#[test]
fn create_url_entry_tar_zst() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("create-url-entry")
        .arg("https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.zst")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(
            r#"{
  "size": 38886,
  "hash": "blake3",
  "digest": "2dedd1652985d33607d1eff62fe9f85f7f53dfce9902cb5de8a8c680cba48081",
  "format": "tar.zst",
  "path": "TODO: specify the appropriate `path` for this artifact",
  "providers": [
    {
      "url": "https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.zst"
    }
  ]
}
"#,
        );
}

#[test]
fn download_only_downloads_and_reuses_cache() -> anyhow::Result<()> {
    let test_env = DotslashTestEnv::try_new()?;
    let dotslash_file = "tests/fixtures/http__tar_gz__print_argv";
    let contents = fs::read_to_string(test_env.current_dir().join(dotslash_file))?;
    let unavailable_file = NamedTempFile::new()?;
    fs::write(
        unavailable_file.path(),
        contents.replace(r#""url":"#, r#""type": "unavailable", "url":"#),
    )?;

    let assert = test_env
        .dotslash_command()
        .arg("--")
        .arg("get-extracted-cache-path")
        .arg(dotslash_file)
        .assert()
        .code(0)
        .stderr_eq("");
    let artifact = Path::new(str::from_utf8(&assert.get_output().stdout)?.trim_end());
    assert!(!artifact.exists());

    test_env
        .dotslash_command()
        .arg("--")
        .arg("download-only")
        .arg(unavailable_file.path())
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'download-only' command failed
caused by: unknown provider type: `unavailable`
",
        );
    assert!(!artifact.exists());

    test_env
        .dotslash_command()
        .arg("--")
        .arg("download-only")
        .arg(dotslash_file)
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq("");
    let metadata = fs::metadata(artifact)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);

    test_env
        .dotslash_command()
        .arg("--")
        .arg("download-only")
        .arg(unavailable_file.path())
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq("");
    assert!(artifact.is_file());

    Ok(())
}

#[test]
fn download_only_no_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("download-only")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'download-only' command failed
caused by: expected exactly one argument but received none
",
        );
}

#[test]
fn download_only_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("download-only")
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .arg("extra_arg")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'download-only' command failed
caused by: expected exactly one argument but received more
",
        );
}

#[test]
fn download_only_platform_override() -> anyhow::Result<()> {
    let test_env = DotslashTestEnv::try_new()?;
    let (target_platform, target_os, target_executable) = if_win_else!(
        ("linux-aarch64", "linux", "print_argv.linux.aarch64"),
        ("windows-x86_64", "windows", "print_argv.windows.x86_64.exe"),
    );
    let url = concat!(
        "https://github.com/zertosh/dotslash_fixtures/raw/",
        "5adea95f2eac6509cad9ca87eb770596a1a21379/pack.tar.gz",
    );
    let artifact_entry = format!(
        r#"{{
  "size": 155817,
  "hash": "blake3",
  "digest": "0a7ec8e59cc9f7266b89db9a3e81558dee49650cc341252b1cab29e60cee7da0",
  "format": "tar.gz",
  "path": "subdir/{target_executable}",
  "providers": [{{"url": "{url}"}}]
}}"#,
    );
    let unavailable_entry = artifact_entry
        .replace(target_executable, "unavailable")
        .replace(r#""url":"#, r#""type": "unavailable", "url":"#);
    let contents = format!(
        r#"#!/usr/bin/env dotslash
{{"name": "foreign", "platforms": {{
  "{target_platform}": {artifact_entry},
  "{target_os}": {unavailable_entry}
}}}}
"#,
    );
    let dotslash_file = NamedTempFile::new()?;
    fs::write(dotslash_file.path(), &contents)?;

    let assert = test_env
        .dotslash_command()
        .args(["--", "get-extracted-cache-path"])
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .code(0)
        .stderr_eq("");
    let artifact = Path::new(str::from_utf8(&assert.get_output().stdout)?.trim_end())
        .with_file_name(target_executable);
    assert!(!artifact.exists());

    test_env
        .dotslash_command()
        .args(["--", "download-only", "--platform", target_platform])
        .arg(dotslash_file.path())
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq("");
    let metadata = fs::metadata(&artifact)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);

    fs::write(
        dotslash_file.path(),
        contents
            .replace(&format!("\"{target_platform}\": {artifact_entry},"), "")
            .replace(&unavailable_entry, &artifact_entry),
    )?;
    for platform in [target_platform, target_os] {
        test_env
            .dotslash_command()
            .args(["--", "download-only"])
            .arg(dotslash_file.path())
            .args(["--platform", platform])
            .assert()
            .code(0)
            .stdout_eq("")
            .stderr_eq("");
    }

    Ok(())
}

#[test]
fn download_only_unavailable_platform() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .args(["--", "download-only", "--platform", "unsupported-x86_64"])
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'download-only' command failed
caused by: platform not supported
caused by: expected platform `unsupported`, `unsupported-x86_64` - but found [..]
",
        );
}

#[test]
fn download_only_platform_argument_errors() {
    for (args, message) in [
        (vec!["--platform"], "expected a value after --platform"),
        (
            vec![
                "--platform",
                "linux-aarch64",
                "--platform",
                "windows-x86_64",
            ],
            "--platform may only be specified once",
        ),
    ] {
        DotslashTestEnv::try_new()
            .unwrap()
            .dotslash_command()
            .args(["--", "download-only"])
            .args(args)
            .assert()
            .code(1)
            .stdout_eq("")
            .stderr_eq(format!(
                "dotslash error: 'download-only' command failed\ncaused by: {message}\n"
            ));
    }
}

//
// "fetch" Command
//

#[test]
fn fetch_simple() -> anyhow::Result<()> {
    let mut test_env = DotslashTestEnv::try_new()?;
    test_env.path_redaction(
        "[ARTIFACT_EXE]",
        "[DOTSLASH_CACHE_DIR]/[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
    );

    let assert = test_env
        .dotslash_command()
        .arg("--")
        .arg("fetch")
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("[ARTIFACT_EXE]\n");

    let artifact = str::from_utf8(&assert.get_output().stdout)?.trim_end();

    let metadata = fs::metadata(artifact)?;
    assert!(metadata.is_file());

    Ok(())
}

#[cfg(unix)]
#[test]
fn fetch_plain_sets_default_executable_permissions() -> anyhow::Result<()> {
    let mut test_env = DotslashTestEnv::try_new()?;
    test_env.path_redaction(
        "[ARTIFACT_EXE]",
        "[DOTSLASH_CACHE_DIR]/[PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
    );

    let assert = test_env
        .dotslash_command()
        .arg("--")
        .arg("fetch")
        .arg("tests/fixtures/http__plain__print_argv")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("[ARTIFACT_EXE]\n");

    let artifact = str::from_utf8(&assert.get_output().stdout)?.trim_end();

    let metadata = fs::metadata(artifact)?;
    assert!(metadata.is_file());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o555);

    Ok(())
}

//
// "get-extracted-cache-path" Command
//

#[test]
fn get_extracted_cache_path_simple() -> anyhow::Result<()> {
    let mut test_env = DotslashTestEnv::try_new()?;
    test_env.path_redaction(
        "[ARTIFACT_EXE]",
        "[DOTSLASH_CACHE_DIR]/[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
    );

    test_env
        .dotslash_command()
        .arg("--")
        .arg("get-extracted-cache-path")
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("[ARTIFACT_EXE]\n");

    Ok(())
}

#[test]
fn get_extracted_cache_path_no_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("get-extracted-cache-path")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'get-extracted-cache-path' command failed
caused by: expected exactly one argument but received none
",
        );
}

#[test]
fn get_extracted_cache_path_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("get-extracted-cache-path")
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .arg("extra_arg")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'get-extracted-cache-path' command failed
caused by: expected exactly one argument but received more
",
        );
}

#[test]
fn get_extracted_cache_path_nonexistent_file() {
    let env = DotslashTestEnv::try_new().unwrap();
    let output = env
        .dotslash_command()
        .arg("--")
        .arg("get-extracted-cache-path")
        .arg("nonexistent_file")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("'get-extracted-cache-path' command failed"));
    assert!(
        stderr.contains("No such file or directory")
            || stderr.contains("cannot find the file")
            || stderr.contains("The system cannot find the file specified")
    );
}

#[test]
fn get_extracted_cache_path_invalid_dotslash_file() -> anyhow::Result<()> {
    let mut invalid_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut invalid_file, b"not a dotslash file")?;

    let env = DotslashTestEnv::try_new()?;
    let output = env
        .dotslash_command()
        .arg("--")
        .arg("get-extracted-cache-path")
        .arg(invalid_file.path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("'get-extracted-cache-path' command failed"));
    assert!(stderr.contains("failed to parse"));

    Ok(())
}

//
// "parse" Command
//

#[test]
fn parse_command_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("tests/fixtures/http__dummy_values.in")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(ci::snapshot_file("http__dummy_values.out"));
}

#[test]
fn parse_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("path/to/dotslash_file")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'parse' command failed
caused by: expected exactly one argument but received more
",
        );
}

#[test]
fn parse_command_non_existent_file() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("fake/path")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'parse' command failed
caused by: failed to read file `fake/path`
caused by: [IO_ERROR_NOT_FOUND]
",
        );
}

//
// "sha256" Command
//

#[test]
fn sha256_command_ok() -> anyhow::Result<()> {
    let tempfile = NamedTempFile::new()?;
    fs::write(tempfile.path(), "DotSlash Rulez!\n")?;

    DotslashTestEnv::try_new()?
        .dotslash_command()
        .arg("--")
        .arg("sha256")
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("52aa28f4f276bdd9a103fcd7f74f97f2bffc52dd816b887952f791b39356b08e\n");

    Ok(())
}
