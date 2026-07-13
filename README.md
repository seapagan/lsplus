# LSPlus - an 'ls' clone written in Rust <!-- omit in toc -->

[![Rust](https://github.com/seapagan/lsplus/actions/workflows/rust.yml/badge.svg)](https://github.com/seapagan/lsplus/actions/workflows/rust.yml)
[![GitHub issues](https://img.shields.io/github/issues/seapagan/lsplus)](https://github.com/seapagan/lsplus/issues)
![Crates.io License](https://img.shields.io/crates/l/lsplus)
[![Crates.io Version](https://img.shields.io/crates/v/lsplus?link=https%3A%2F%2Fcrates.io%2Fcrates%2Flsplus)](https://crates.io/crates/lsplus)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/73f67c2ab44548298e0660ca73308729)](https://app.codacy.com/gh/seapagan/lsplus/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![Build Docs](https://github.com/seapagan/lsplus/actions/workflows/gh-pages.yml/badge.svg)](https://github.com/seapagan/lsplus/actions/workflows/gh-pages.yml)

LSPlus is a functional Unix `ls` clone written in Rust. It was originally built
as a Rust learning project, but has become a full-featured clone that i use on
all my machines as standard, replacing GNU ls.

> [!NOTE]
>
> Currently much of the development of this tool is now done using AI coding
> tools - primarily `GPT-5.5` running in [Codex](https://openai.com/codex/) for
> implementation, with `DeepSeek` (Running in [Oh-My-Pi](https://omp.sh/)) and
> `GLM 5.2` (Running in [Claude Code](https://claude.com/product/claude-code))
> performing independent local reviews for each PR. New non-trivial features are
> first iteratively discussed and planned with `ChatGPT` at `Pro` level
> thinking. All of the above run side by side in tabs using
> [Zellij](https://zellij.dev/) for multiplexing and session persistence.

![lsp output](./docs/src/images/screenshot.png)

<!-- vim-markdown-toc GFM -->

- [Compatibility](#compatibility)
- [Nerd Fonts](#nerd-fonts)
- [Installation](#installation)
  - [Download a Binary](#download-a-binary)
  - [Using Cargo](#using-cargo)
  - [Using "cargo-binstall"](#using-cargo-binstall)
  - [From Source](#from-source)
- [Usage](#usage)
  - [Fuzzy Time](#fuzzy-time)
  - [Icons](#icons)
  - [Compatibility Mode](#compatibility-mode)
  - [Configuration File](#configuration-file)
  - [Aliases](#aliases)
- [Development](#development)
- [Future Plans](#future-plans)

<!-- vim-markdown-toc -->

## Compatibility

LSPlus supports Linux, macOS, and Windows. Windows uses native file attributes,
`PATHEXT` command classification, and junction-aware traversal.

## Nerd Fonts

To display the folder and file icons, you need to first install a 'Nerd Font'
for your terminal. You can find a great selection of Nerd Fonts from the
[Nerd Fonts website](https://www.nerdfonts.com/)

My personal favourite is `MesoLG Nerd Font`, but there are many others to choose
from. You will also need to set up your terminal to use that font.

If you **DO NOT** want to install a Nerd Font, pass the `--no-icons` switch to
the program (or `no_icons=true` in the config file).

## Installation

### Download a Binary

Download the latest Linux, macOS, or Windows archive from the [release
page](https://github.com/seapagan/lsplus/releases/latest). Unpack it and move
`lsp` (or `lsp.exe` on Windows) into a directory on your `PATH`. On Unix,
make it executable if needed.

These binaries are auto-generated for each release.

### Using Cargo

If you have rust installed, you can install the latest release of this package,
using the following command:

```bash
cargo install lsplus
```

This will install the `lsp` binary into your `~/.cargo/bin` directory. Make
sure that this directory is in your `PATH` environment variable so that you
can run the `lsp` command from anywhere. See the `binstall` section below for a
quicker way to install using cargo.

### Using "cargo-binstall"

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation)
installed, you can install `lsplus` more quickly and easily using:

```bash
cargo binstall lsplus
```

This will install the latest binary into your cargo bin folder without needing
compilation.

### From Source

You can also install the package from the GitHub repository by running the
following command:

```bash
cargo install --git https://github.com/seapagan/lsplus.git
```

## Usage

Run this command in your terminal to list files in the current directory:

```sh
lsp <options> <path | file>
```

Options and paths are optional. With no path, `lsp` lists the current
directory. With no options, it uses defaults similar to `ls`.

Currently, only a sub-set of the standard `ls` options are supported. These are:

- `-a` / `--all` - Show hidden files
- `-A` / `--almost-all` - Show hidden files, but don't show `.` and `..`
- `-p` / `--slash-dirs` - Append a '/' to directories
- `--file-type` - Append type indicators except `*` for executables
- `-F` / `--classify` - Append type indicators, including `*` for executables
- `--no-indicators` - Disable file type indicators
- `-l` / `--long` - Show long format listing
- `--header` - Show a title row in long-format output
- `--permissions <MODE>` - Select long-format permission display:
  `symbolic`, `octal`, `both`, or `none`
- `--attributes <MODE>` - Select `long`, `short`, or `minimal` Windows attribute display
- `-h` / `--human-readable` - Human readable file sizes using powers of 1024
- `--si` - Human readable file sizes using powers of 1000
- `-R` / `--recursive` - List subdirectories recursively
- `-D` / `--sort-dirs` - Sort directories first
- `-I` / `--gitignore` - Dim entries matched by Git ignore rules
- `-N` / `--no-color` - Disable colored and styled output
- `--no-permission-colors` - Disable long-format file type character and
  permission colors
- `--no-time-gradient` - Use the fixed long-format timestamp color
- `--no-size-colors` - Disable long-format large-size colors
- `--no-icons` - don't show file or folder icons
- `-V` / `--version` - Print version information and exit
- `-Z` / `--fuzzy-time` - Show fuzzy time for file modification times

You can combine the short options together, e.g. `-laph` will show a long format
listing with hidden files, append a '/' to directories, and show human-readable
file sizes.

Use the `--help` option to see the full list of options.

Use `-R` or `--recursive` to print GNU-style recursive directory sections.
Quoted wildcard or filename operands filter matching entries while still
walking subdirectories:

```sh
lsp -R '*.rs'
lsp -R 'src/*.rs'
```

Bare filename operands such as `main.rs` search for that basename below the
current directory. Prefixed literal paths such as `src/main.rs` remain exact
path operands. When `--level` is used with a recursive filter, no-match
diagnostics apply to matches visible within that depth limit.

Quote or escape wildcard patterns in shells such as zsh. Otherwise the shell
may expand or reject the pattern before `lsp` starts, which is the same
limitation GNU `ls` has for unquoted wildcards.

The indicator characters are:

- `/` for directories
- `@` for symlinks
- `|` for FIFOs
- `=` for sockets
- `*` for executables, but only with `-F` / `--classify`

In long format, native mode omits the symlink `@` marker because `name ->
target` and the symlink styling already make the type clear. This also matches
GNU `ls`, which does not append `@` to symlink names in long format.

When `-I` is enabled, `lsp` checks the same ignore sources Git normally uses:
merged `.gitignore` files in the worktree, `.git/info/exclude`, and the
configured global Git excludes file.

Styled output is enabled automatically when writing to a terminal. Captured,
piped, and redirected output is plain by default. You can also disable styled
output explicitly with `--no-color`, `no_color = true` in the config file, or
the `NO_COLOR` environment variable.

Long-format output shows symbolic permissions by default. Use
`--permissions octal` to replace them with octal permission bits,
`--permissions both` to add octal bits after the symbolic field, or
`--permissions none` to omit permission fields.

On Windows, symbolic permission display is replaced by an `Attributes` column.
`--attributes long` is the default and shows readable names.
`--attributes short` uses a fixed-position 17-character prefix in
`RHSATPCONEIVBXQGF` order; residual unknown bits append an
`Unknown(0xXXXXXXXX)` suffix. The `X` position represents `EA`, including the
aliased `RecallOnOpen` bit, while `F` represents `RecallOnDataAccess`.
`--attributes minimal` shows only the classic `RHSA` attributes and shortens
the column header to `Attr`.
`--permissions none` omits the column. `--permissions octal` and
`--permissions both` are unsupported with long output.

The attribute setting has no effect on short-format output. It is accepted but
ignored on Linux and macOS, where permission rendering remains unchanged.

Long-format output colors permission bits, timestamp freshness, and large file
sizes by default. You can adjust those accents independently with
`--no-permission-colors`, `--no-time-gradient`, `--no-size-colors`, or the
matching `permission_colors = false`, `time_gradient = false`, and
`size_colors = false` config options.

Timestamp colors adapt to terminal color capability. Truecolor terminals use a
smooth age gradient and 256-color terminals use a stepped fallback to
distinguish files newer than a day, week, month, and year. Basic ANSI terminals
use named yellow styling. Disabling `time_gradient` keeps normal timestamps on
the original fixed timestamp color. Future-dated timestamps stay red to make
clock-skewed files stand out.

### Fuzzy Time

The `-Z` option will show a fuzzy time for file modification times. This will
show the time in a human-readable format, e.g. '2 hours ago', 'yesterday', etc.

![fuzzy date output](./docs/src/images/screenshot3.png)

### Icons

`lsp` shows icons for folders, files, and links. The current mappings cover
common names and extensions. Open an issue or PR if you want another icon.

You can disable the icons by using the `--no-icons` option.

### Compatibility Mode

`lsp` has two CLI modes:

- `native` - the default `lsplus` command-line interface
- `gnu` - a GNU `ls` compatibility mode intended for aliases and scripts

You can enable GNU compatibility mode in either of these ways:

```toml
compat_mode = "gnu"
```

or:

```sh
LSP_COMPAT_MODE=gnu lsp
```

The `LSP_COMPAT_MODE` environment variable takes precedence over the config
file.

At the moment, compatibility mode only changes the CLI surface and help output.
It does not yet implement the missing GNU meanings for the conflicting short
flags `-D`, `-I`, `-N`, and `-Z`; those flags are reserved in `gnu` mode and
will error until their GNU behavior is implemented.

The current `lsplus` features behind those four native short flags are still
available in `gnu` mode through their long forms only:

- `--group-directories-first` (replaces the original `--sort-dirs`)
- `--gitignore`
- `--no-color`
- `--fuzzy-time`

GNU indicator options are also available in `gnu` mode:

- `-p` / `--indicator-style=slash`
- `--file-type` / `--indicator-style=file-type`
- `-F` / `--indicator-style=classify`
- `--indicator-style=none`

### Configuration File

Put options in the config file to apply them to each run instead of passing them
on the command line. See the relevant section on the
[website](https://seapagan.github.io/lsplus/config.html) for full details.

Set `LSP_CONFIG_FILE` to use a specific config file instead of the platform
default path. An unset or empty value keeps the normal platform default.

### Aliases

The `lsp` command can be aliased to `ls` by adding the following line to your
`.bashrc`, `.zshrc` or similar file:

```sh
alias ls='lsp'
```

If you want that alias to behave more like GNU `ls`, enable `gnu`
compatibility mode in your config file or set `LSP_COMPAT_MODE=gnu` in your
shell environment.

You will need to restart your shell or source your configuration file for the
alias to take effect.

The example below shows an alias for ls that uses many of the current options:

```sh
alias ls='lsp -laph'
```

This will show a long format listing with hidden files, append a '/' to
directories, and show human readable file sizes, as in the image above.

On Windows, PowerShell users can add Linux-like commands to their PowerShell
profile (`$PROFILE`):

```powershell
Set-Alias -Name ls -Value lsp -Force
function ll { lsp -l @args }
```

Restart PowerShell or source the profile for the commands to take effect.

Command Prompt users can create equivalent macros with `doskey`:

```bat
doskey ls=lsp $*
doskey ll=lsp -l $*
```

To make these macros persistent, store them without the `doskey` prefix in
`%USERPROFILE%\doskey.macros`:

```text
ls=lsp $*
ll=lsp -l $*
```

Then configure Command Prompt to load the file automatically:

```bat
reg add "HKCU\Software\Microsoft\Command Processor" /v AutoRun /t REG_EXPAND_SZ /d "doskey /macrofile=\"^%USERPROFILE^%\doskey.macros\"" /f
```

This replaces any existing `AutoRun` command, so check it first:

```bat
reg query "HKCU\Software\Microsoft\Command Processor" /v AutoRun
```

If an `AutoRun` value already exists, copy only the command text after its type
(`REG_EXPAND_SZ` or `REG_SZ`) from the `reg query` output, not the full output.
Escape each `"` in that command as `\"`, then replace `existing command` below
to chain the commands with `&`:

```bat
reg add "HKCU\Software\Microsoft\Command Processor" /v AutoRun /t REG_EXPAND_SZ /d "doskey /macrofile=\"^%USERPROFILE^%\doskey.macros\" & existing command" /f
```

## Development

This repo uses `husky-rs` to manage local Git hooks for contributors. After
cloning the repo, run:

```bash
cargo test
```

That installs the versioned hooks from `.husky/` for this checkout. The hooks
currently run:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` on `pre-push`

Optional development tooling is provided through Cargo subcommands.

For task aliases and dependency checks, install:

```bash
cargo install cargo-make
cargo install cargo-audit
cargo install cargo-deny
cargo install --locked --version 1.26.1 zizmor
```

Audit the GitHub Actions workflows locally with `cargo make zizmor`. The
command runs offline by default; set `GH_TOKEN`, `GITHUB_TOKEN`, or
`ZIZMOR_GITHUB_TOKEN` to enable online audits. Keep the installed zizmor
version aligned with the version configured in the GitHub Actions security
workflow.

The coverage aliases, `cargo make test` and `cargo make test-html`, also
require:

```bash
cargo install cargo-nextest
cargo install cargo-llvm-cov
```

On Linux, install `cargo-xwin` to cross-check the Windows MSVC target:

```bash
cargo install cargo-xwin
rustup target add x86_64-pc-windows-msvc
```

Run all native Unix and Windows cross-target verification checks with:

```bash
cargo make verify
```

Use `cargo make verify-unix` or `cargo make verify-windows` to run either set
of checks independently. These Cargo Make verification tasks are Linux-only:
`verify-windows` depends on `cargo-xwin`, whose Windows SDK downloads are not
supported on Windows hosts. The Windows checks compile, lint, and build only;
Windows CI remains responsible for the native test suite.

On Windows, run the native checks directly:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --all-targets --all-features
```

## Future Plans

See [TODO](./TODO.md) for planned work.
