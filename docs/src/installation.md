# Installation

## Download a Binary

Download the latest Linux, macOS, or Windows archive from the [release
page](https://github.com/seapagan/lsplus/releases/latest). Unpack it and move
`lsp` (or `lsp.exe` on Windows) into a directory on your `PATH`. On Unix,
make it executable if needed.

These binaries are auto-generated for each release.

## Using Cargo

If you have rust installed, you can install the latest release of this package,
using the following command:

```bash
cargo install lsplus
```

This will install the `lsp` binary into:

- Linux and macOS: `~/.cargo/bin`
- Windows: `%USERPROFILE%\\.cargo\\bin`

Make sure this directory is in your `PATH` environment variable so that you
can run the `lsp` command from anywhere.

## From Source

Install the package from the GitHub repository with:

```bash
cargo install --git https://github.com/seapagan/lsplus.git
```

This uses unreleased commits from the default branch.
