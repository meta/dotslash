---
sidebar_position: 12
---

# DotSlash File Schema

This article explains the requirements for a DotSlash file, so we will use a
working example for reference. The following is a DotSlash file for the `hermes`
CLI taken from the
[v0.12.0 release on GitHub](https://github.com/facebook/hermes/releases/tag/v0.12.0).
Incidentally, the entries for `"macos-x86_64"` and `"macos-aarch64"` are
identical because on macOS, Hermes is provided as a
[Universal Binary](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary),
so the same artifact works for both architectures:

```json
#!/usr/bin/env dotslash

// This DotSlash file represents the hermes CLI across multiple platforms for
// the v0.12.0 release of Hermes: https://github.com/facebook/hermes/releases/tag/v0.12.0

{
  "name": "hermes",
  "platforms": {
    "macos-x86_64": {
      "size": 10600817,
      "hash": "blake3",
      "digest": "25f984911f199f9229ca0327c52700fa9a8db9aefe95e84f91ba6be69902436a",
      "format": "tar.gz",
      "path": "hermes",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-darwin-v0.12.0.tar.gz"
        },
        {
          "type": "github-release",
          "repo": "facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-darwin-v0.12.0.tar.gz"
        }
      ],
    },
    "macos-aarch64": {
      "size": 10600817,
      "hash": "blake3",
      "digest": "25f984911f199f9229ca0327c52700fa9a8db9aefe95e84f91ba6be69902436a",
      "format": "tar.gz",
      "path": "hermes",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-darwin-v0.12.0.tar.gz"
        },
        {
          "type": "github-release",
          "repo": "facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-darwin-v0.12.0.tar.gz"
        }
      ],
    },
    "linux-x86_64": {
      "size": 47099598,
      "hash": "blake3",
      "digest": "8d2c1bcefc2ce6e278167495810c2437e8050780ebb4da567811f1d754ad198c",
      "format": "tar.gz",
      "path": "hermes",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-linux-v0.12.0.tar.gz"
        },
        {
          "type": "github-release",
          "repo": "facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-linux-v0.12.0.tar.gz"
        }
      ],
    },
    "windows-x86_64": {
      "size": 17456100,
      "hash": "blake3",
      "digest": "7efee4f92a05e34ccfa7c21c7a05f939d8b724bc802423d618db22efb83bfe1b",
      "format": "tar.gz",
      "path": "hermes.exe",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-windows-v0.12.0.tgz"
        },
        {
          "type": "github-release",
          "repo": "facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-windows-v0.12.0.tgz"
        }
      ],
    }
  }
}
```

The general structure of a DotSlash file is:

```json
#!/usr/bin/env dotslash

{
  "name": /* string */,
  "platforms": /* map */,
}
```

That is, the file **must** start with `#!/usr/bin/env dotslash` followed
immediately by either `\n` or `\r\n`. The required "shebang header" must be
followed by a JSON payload, which is required to be a map with the following
properties:

- `"name"` must be a string that represents the name of the executable (it often
  matches the name of the DotSlash file)
- `"platforms"` must be a map of supported platforms to artifacts

:::tip

The JSON payload in a DotSlash file is parsed with a
[lenient JSON parser](https://crates.io/crates/jsonc-parser) that allows for
trailing commas as well as `//` and `/*`-style comments.

:::

The keys in the `"platforms"` map take on a format inspired by
[Clang triples](https://clang.llvm.org/docs/CrossCompilation.html). Today,
DotSlash supports the following keys in the `"platforms"` map:

- `linux-aarch64`
- `linux-x86_64`
- `macos-aarch64`
- `macos-x86_64`
- `windows-aarch64`
- `windows-x86_64`

When `dotslash` runs a DotSlash file, it considers only the entry in
`"platforms"` that corresponds to the target platform for which that version of
`dotslash` was built. The schema of such a _platform entry_ that specifies the
artifact to fetch is as follows:

```json
{
  "size": /* size of the artifact in bytes (nonnegative integer) */,
  "hash": /* name of hash algorithm: either "sha256" or "blake3" */,
  "digest": /* artifact digest as a lowercase hex string */,
  "providers": /* array of providers */,
  "format": /* recognized format, such as "tar.gz"; see list below */,
  "path": /* filename or path within an archive */,
  "arg0": /* arg0 behavior, see below */,
  "readonly": /* `false` disables `chmod -R -w` on the unpacked artifact */,
}
```

A platform entry must specify several fields:

- The `size`, `hash`, and `digest` are used to verify that the fetched artifact
  has the expected contents. When DotSlash fetches a blob using a provider, it
  will not decompress or run it if the `size`/`hash`/`digest` does not match
  what is in the DotSlash file.
- The list of `providers` specifies how/where DotSlash can try to acquire the
  artifact. A provider will get the opportunity to write the artifact to a
  temporary file on disk where DotSlash will verify it before proceeding.
- In the event of a successful fetch, the `format` and `path` entries determine
  how the artifact should be decompressed and, in the case of an archive, the
  entry within the archive to execute when the DotSlash file is run.

Let's discuss to role of each of set of parameters in more detail.

## Verification

To ensure that the artifact that was fetched matches what the author of the
DotSlash file intended, once the artifact is on disk, its content is verified
against the `size`, `hash`, and `digest` parameters.

Today, the only acceptable values for `hash` are `"sha256"` and `"blake3"`,
which correspond to the SHA-256 and BLAKE3 hash functions, respectively.

From the command line, you can use DotSlash's "hidden" subcommands to compute
the hash for a file on disk, which is handy if you do not have `shasum -a 256`
or `b3sum` readily available on your `$PATH`:

```shell
$ echo 'DotSlash Rulez!' > /tmp/dotslash-example.txt
$ dotslash -- sha256 /tmp/dotslash-example.txt
52aa28f4f276bdd9a103fcd7f74f97f2bffc52dd816b887952f791b39356b08e
$ dotslash -- b3sum /tmp/dotslash-example.txt
824ecff042b9ac68a33ea5ee027379a09f3da1d5a47f29fc52072809a204db87
```

## Providers

The `providers` parameter must be a list even though in practice, it often
contains only one value.

A provider is specified via a JSON object that has a `"type"` field to tell
DotSlash what type of provider it is, though if unspecified, `"http"` is
assumed. Each provider defines its own schema with respect to the other fields
that must be specified on the JSON object.

Currently, DotSlash incldues three providers:

- HTTP Provider: `"type": "http"`
- GitHub Release Provider: `"type": "github-release"`
- S3 Provider: `"type": "s3"`
- GCS Provider: `"type": "gcs"`

At the time of this writing, there is no way to
add custom providers without forking DotSlash.

Each provider in the `providers` list will be tried, in order by default, to
fetch the artifact, until one succeeds. The provider type need not be unique
within a list, e.g., the HTTP Provider can be specified multiple times with
different values for `"url"`. Though note that the `size`/`hash`/`digest` are
specified _independently_ of the providers, so all providers must yield the same
artifact.

The order in which providers are tried can be randomized by adding
`providers_order: "weighted-random"` under the artifact entry. This will
randomize the order in which providers are tried, using an optional `weight` for
each provider to bias the selection. For example:

```json
{
  ...
  "format": "tar.gz",
  "path": "hermes",
  "providers": [
    {"url": "https://primary.example.com/hermes.tar.gz", "weight": 3},
    {"url": "https://mirror1.example.com/hermes.tar.gz", "weight": 1},
    {"url": "https://mirror2.example.com/hermes.tar.gz", "weight": 1},
    {"url": "https://mirror3.example.com/hermes.tar.gz", "weight": 1}
  ],
  "providers_order": "weighted-random"
}
```

In this example, the primary provider is more likely to be tried first, but the
remaining three mirrors are equally likely to be tried next. Providers are not
retried if they fail, so the first successful provider wins.

The weight must be an integer value greater than or equal to `1`. If weight is
not specified for a provider, it defaults to `1`.

### HTTP Provider

As shown in the Hermes example, the only required field when using the HTTP
provider is the `"url"` that specifies the URL from which to fetch the artifact
via `HTTP GET` using `curl`.

:::tip

`curl` requests from DotSlash include a custom user-agent that looks something
like this:

```shell
Mozilla/5.0 (compatible; DotSlash/0.1.0; +https://dotslash-cli.com)
```

:::

#### Authentication

`curl` is invoked with `--netrc-optional`, so URLs that require HTTP basic
authentication work when the host has an entry in your `~/.netrc` file:

```text
machine artifacts.example.com login myuser password mytoken
```

On Windows, curl also reads `%USERPROFILE%\_netrc` and `%USERPROFILE%\.netrc`.
Hosts without an entry are fetched without credentials. Credentials are never
stored in the DotSlash file.

Note that in order to facilitate creating a DotSlash file by hand, you can use
DotSlash's `create-url-entry` subcommand to generate the boilerplate for a
platform entry based on a URL as follows:

```shell
$ dotslash -- create-url-entry https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-linux-v0.12.0.tar.gz
{
  "size": 47099598,
  "hash": "blake3",
  "digest": "8d2c1bcefc2ce6e278167495810c2437e8050780ebb4da567811f1d754ad198c",
  "format": "tar.gz",
  "path": "TODO: specify the appropriate `path` for this artifact",
  "providers": [
    {
      "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-linux-v0.12.0.tar.gz"
    }
  ]
}
```

That is, `create-url-entry` will download the file at the specified URL, compute
its size in bytes as well as its BLAKE3 hash, and then print "approximately" the
JSON needed to represent this artifact in a DotSlash file. The following caveats
apply to the output:

- The default value of `"path"` is a `TODO` because `create-url-entry` does not
  know which path within the `.tar.gz` archive is intended to be used in the
  DotSlash file you are trying to generate.
- In this case, `"format"` happens to be the appropriate value (`"tar.gz"`)
  because DotSlash guessed correctly based on the suffix of the URL; however,
  this logic is based on a set of heuristics, so it could choose the wrong
  value. Be sure to check the value of `"format"` when using the output of this
  tool.

Bear in mind that if the `"url"` is not publicly accessible (or is not
accessible to the user if, say, it can only be accessed while on a VPN), the
`curl` request will fail. In such cases, a different type of provider may be the
solution.

### GitHub Release Provider

The GitHub Release Provider facilitates fetching artifacts that are published as
part of a release in a GitHub repository. Unlike the HTTP Provider, it uses
[the GitHub CLI (`gh`)](https://cli.github.com) to fetch an artifact. Users are
responsible for installing `gh` and making it available on their `$PATH`.

The primary advantage of using this provider over the HTTP provider is that it
can fetch artifacts from non-public GitHub URLs, such as private repositories or
repositories hosted on a GitHub Enterprise instance, so long as the user has
authenticated via `gh auth` so the CLI can read from those repositories.

An instance of the provider such as:

```json
{
  "type": "github-release",
  "repo": "facebook/hermes",
  "tag": "v0.12.0",
  "name": "hermes-cli-linux-v0.12.0.tar.gz"
}
```

gets translated into the following command in order to do the fetch:

```shell
gh release download v0.12.0 \
  --repo facebook/hermes \
  --pattern 'hermes-cli-linux-v0\.12\.0\.tar\.gz' \
  --output TEMPFILE_IN_DOTSLASH_CACHE
```

Note that if the `facebook/hermes` repo were part of a GitHub Enterprise
instance hosted on `example.com`, the JSON in the DotSlash file would have to
be:

```json
{
  "type": "github-release",
  // Note the change to this value!
  "repo": "example.com/facebook/hermes",
  "tag": "v0.12.0",
  "name": "hermes-cli-linux-v0.12.0.tar.gz"
}
```

And the `--repo` value passed to `gh` would also be changed, accordingly.

### S3 Provider

The S3 provider allows fetching artifacts from S3 (or any other compatible
object store) using the [`aws` CLI](https://aws.amazon.com/cli/). The advantage
of the S3 provider is that it allows you to retrieve artifacts from S3 buckets
that are not accessible via HTTP (e.g. private).

```json
{
  "type": "s3",
  "repo": "example",
  "key": "key",
  "region": "us-west-2"
}
```

gets translated into the following command in order to do the fetch:

```shell
aws s3 cp --region us-weset-2 s3://example/key TEMPFILE_IN_DOTSLASH_CACHE
```

Note: the "region" key is optional. By default, your default AWS region will be
used.

You will need to have the `aws` CLI configured to pick up the required
credentials. Either via its built-in credentials mechanism, or wrapping the
`dotslash` invocation tools such as
[`aws-vault`](https://github.com/99designs/aws-vault).

## Artifact Format

Although it may appear that `format` can be an arbitrary file extension,
DotSlash recognizes a **fixed set of values** that have the following
properties:

| `format`  | Archive? | Decompress? |
| --------- | -------- | ----------- |
| `tar.bz2` | yes      | bzip2       |
| `tar.gz`  | yes      | gzip        |
| `tar.xz`  | yes      | xz          |
| `tar.zst` | yes      | zstd        |
| `tar`     | yes      | _none_      |
| `zip`     | yes      | zip         |
| `bz2`     | no       | bzip2       |
| `gz`      | no       | gzip        |
| `xz`      | no       | xz          |
| `zst`     | no       | zstd        |
| _omitted_ | no       | _none_      |

An artifact is either an _archive_ (such as a `.tar` file) or a _single file_.

### Path

Because the `path` identifies the file to execute within the unpacked artifact's
directory in the DotSlash cache, the `format` determines how the `path` will be
interpreted:

- In the case of an _archive_, it will be unpacked in its own directory in the
  DotSlash cache. The `path` specifies the relative path within that directory
  as the file to run when the DotSlash file is executed.
- In the case of a _single file_, a new, empty directory will also be written in
  the DotSlash cache. The `path` specifies the relative path where the file
  should be written in that directory.

DotSlash has strict requirements for the `path` parameter:

:::important

`path` **must** be specified as a _normalized, relative UNIX path_, even for
Windows artifacts.

:::

This restriction is designed to prevent referencing files "outside" the cache
via either relative or absolute paths. To reduce ambiguity, DotSlash rejects a
`path` that contains a backslash (`\`) even though it is an allowed character
for a "normal" path component on UNIX.

These are **valid values** for `path`:

- `buck2`
- `buck2.exe`
- `path/to/buck2`
- `path/to/buck2.exe`

These are **invalid values** for `path`:

- `path\to\buck2.exe` (contains `\`)
- `/usr/local/bin/buck2` (absolute path: not relative)
- `./buck2` (contains current directory component: not normalized)
- `../buck2` (contains parent directory component: not normalized)
- `buck2/` (slash at end: not normalized)
- `C:\Tools\buck2.exe` (contains `\`)
- `C:/Tools/buck2.exe` ([only on Windows] contains prefix component: not
  relative)

### Format

The `format` describes how the artifact is packaged, which in turn defines how
DotSlash will unpack it.

:::warning

To reiterate, `format` cannot be an arbitrary file extension. For example, if
you have a gzipped-tarball that is served from a URL ending with `.tgz`, you
must still classify it using `"format": "tar.gz"` in the DotSlash file because
`tgz` is not a recognized value for `format`.

:::

DotSlash also supports artifacts that are compressed with either gzip or
[zstd](https://facebook.github.io/zstd/). For artifacts that are compressed
archives, they will be decompressed before they are unzipped. Note that DotSlash
includes its own implementations of gzip and zstd implemented in Rust rather
than relying on a an implementation of `gunzip` or `zstd` on the user's `$PATH`.

Looking at the `hermes` example above:

- Once it is fetched, the artifact will be decompressed as if it were a
  `.tar.gz` file.
- Within the decompressed archive, the `hermes` entry should be executed, except
  on Windows, in which case the `hermes.exe` entry should be used instead.

At Meta, we have found compression to be a win, but if for some reason you
prefer to fetch your executable as an uncompressed single file, you can omit the
`"format"` field, but `"path"` is still required.

For single-file artifacts on Unix, if the fetched file has no executable bits,
DotSlash makes the cached file executable with mode `0555` (`r-xr-xr-x`).

## Arg0

There is an optional `arg0` field on an artifact entry. It defaults to
`"dotslash-file"` on Unix, and to `"underlying-executable"` unconditionally on
Windows. It specifies what is passed as the zeroth argument (`argv[0]`) to the
underlying process. The options are:

- `"dotslash-file"` - This forwards the zeroth argument of the outer `dotslash`
  invocation (in other words, the DotSlash file). This can be useful for
  executables that will use the zeroth argument to modify their behavior or
  behave as different commands, such as `clang` or GNU coreutils. This is the
  default on Unix.
- `"underlying-executable"` - This passes the path to the executable in the
  DotSlash cache as the zeroth argument. This can be useful for executables that
  will use `argv[0]` to locate the path of related files in the same archive, or
  parse trailing data attached to the executable file. This is the unconditional
  default on Windows.

This option has no effect on Windows.

## Readonly

There is an optional `readonly` boolean field on an artifact entry. It is not
intended to be used (it defaults to `true`), but is provided as an escape hatch,
if necessary.

After DotSlash decompresses an artifact in a temporary folder, it marks all of
the entries in the folder
[read-only](https://doc.rust-lang.org/std/fs/struct.Permissions.html#method.set_readonly)
before moving it into its final location in the cache. The idea is that
executables should not be modifying the files in the cache, as that sort of
behavior increases the likelihood of non-determinism.

Of course, it is possible that DotSlash is used to deliver an executable that
may do something like write to a dotfile alongside the executable when it is
run. By default, this write will fail when the executable is run via DotSlash
because the folder is read-only. Ideally, the executable would be redesigned to
write to `$XDG_STATE_HOME` or whatever is appropriate, but the author of the
DotSlash file may not be in a position to change that.

In this case, setting `readonly: false` will disable the logic that marks all of
the entries in the temporary folder read-only before it is moved to its final
location in the cache.
