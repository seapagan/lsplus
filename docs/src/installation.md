# Installation

To install the latest release of this package, you can use the following command:

## Download a Binary

For Linux and Mac you can download the latest binary files from the [release
page](https://github.com/seapagan/lsplus/releases/latest). Unpack the archive
and move the single file `lsp` to somewhere in your path so it can be located.
Ensure it is set as executable (though it already should be).

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

You can also install the package from the GitHub repository by running the
following command:

```bash
cargo install --git https://github.com/seapagan/lsplus.git
```

This may allow you to access the latest features and bug fixes that have not
yet been released.
