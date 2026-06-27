# Usage

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
- `-h` / `--human-readable` - Human readable file sizes using powers of 1024
- `--si` - Human readable file sizes using powers of 1000
- `-R` / `--recursive` - List subdirectories recursively
- `--tree` - Show a long-format directory tree
- `--level <N>` - Limit tree output depth
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

When listing multiple directory operands, or a mix of files and directories,
`lsp` prints file operands first and labels each directory section with a
`path:` header. A single non-recursive directory keeps the compact output shape
without a header.

Use `-R` or `--recursive` to print GNU-style recursive directory sections.
Recursive output is unlimited unless you pass `--level <N>`. Use `--tree` for
long-format tree output. Tree output implies `--long`, uses a default depth of
`2`, and can be limited with `--level <N>`. `--tree` and `--recursive` are
mutually exclusive.

The indicator characters are:

- `/` for directories
- `@` for symlinks
- `|` for FIFOs
- `=` for sockets
- `*` for executables, but only with `-F` / `--classify`

In long format, native mode omits the symlink `@` marker because `name ->
target` and the symlink styling already make the type clear. This also matches
GNU `ls`, which does not append `@` to symlink names in long format.

Styled output is enabled automatically when writing to a terminal. Captured,
piped, and redirected output is plain by default. You can also disable styled
output explicitly with `--no-color`, `no_color = true` in the config file, or
the `NO_COLOR` environment variable.

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

When `-I` is enabled, `lsp` checks the same ignore sources Git normally uses:
merged `.gitignore` files in the worktree, `.git/info/exclude`, and the
configured global Git excludes file.

## Compatibility Mode

`lsp` has two CLI modes:

- `native` - the default `lsplus` command-line interface
- `gnu` - a GNU `ls` compatibility mode intended for aliases and scripts

You can enable GNU compatibility mode by setting `compat_mode = "gnu"` in the
config file or by setting `LSP_COMPAT_MODE=gnu` in the environment. The
environment variable takes precedence over the config file.

At the moment, `gnu` mode changes the CLI surface and help output only. It does
not yet implement the missing GNU meanings for the conflicting short flags
`-D`, `-I`, `-N`, and `-Z`; those flags are reserved in `gnu` mode and will
error until their GNU behavior is implemented.

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

## Fuzzy Time

The `-Z` option shows file modification times in a human-readable format, e.g.
'2 hours ago', 'yesterday', etc.

![fuzzy date output](./images/screenshot3.png)

## Icons

`lsp` shows icons for folders, files, and links. The current mappings cover
common names and extensions. Open an issue or PR if you want another icon.

Disable icons with the `--no-icons` option.

## Aliases

Add this line to `.bashrc`, `.zshrc`, or a similar file to alias `ls` to `lsp`:

```sh
alias ls='lsp'
```

If you want that alias to behave more like GNU `ls`, enable `gnu`
compatibility mode in your config file or set `LSP_COMPAT_MODE=gnu` in your
shell environment.

Restart your shell or source your configuration file to load the alias.

This alias enables several common options:

```sh
alias ll='lsp -laph'
```

This shows a long-format listing with hidden files, appends `/` to directories,
and shows human-readable file sizes.

Set default options in the configuration file.

![lsp output](./images/screenshot.png)

Add `-D` in native mode, or `--group-directories-first` in GNU mode, to sort
directories first:

![lsp output](./images/screenshot2.png)
