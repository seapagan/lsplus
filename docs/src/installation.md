# Installation

## Download a Binary

Download the latest Linux or macOS archive from the [release
page](https://github.com/seapagan/lsplus/releases/latest). Unpack it, move
`lsp` into a directory on your `PATH`, and make it executable if needed.

These binaries are auto-generated for each release.

## Using Cargo

If you have rust installed, you can install the latest release of this package,
using the following command:

```bash
cargo install lsplus
```

This will install the `lsp` binary into your `~/.cargo/bin` directory. Make
sure that this directory is in your `PATH` environment variable so that you
can run the `lsp` command from anywhere.

## From Source

Install the package from the GitHub repository with:

```bash
cargo install --git https://github.com/seapagan/lsplus.git
```

This uses unreleased commits from the default branch.
