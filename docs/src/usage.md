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
- `--header` - Show a title row in long-format output
- `--permissions <MODE>` - Select long-format permission display:
  `symbolic`, `octal`, `both`, or `none`
- `--attributes <MODE>` - Select `long` or `short` Windows attribute display
- `-h` / `--human-readable` - Human readable file sizes using powers of 1024
- `--si` - Human readable file sizes using powers of 1000
- `-R` / `--recursive` - List subdirectories recursively
- `--tree` - Show a long-format directory tree
- `--level <N>` - Limit recursive or tree output to visible entry depth
- `--prune-noisy-dirs` - Skip descending into common noisy directories
- `--prune-dir <NAME>` - Skip descending into matching directory basenames
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

In both recursive and tree output, `--level <N>` counts visible entry levels
below each operand. For example, `--level 1` shows only entries directly under
the requested directory, while `--level 2` also shows grandchildren.

Use `--prune-noisy-dirs` with recursive or tree output to list common noisy
directories in their parent but skip their descendants. The built-in preset
matches `.git`, `.hg`, `.svn`, `node_modules`, and `__pycache__` by exact
basename:

```sh
lsp -R --prune-noisy-dirs project
lsp --tree --prune-noisy-dirs project
```

Use `--prune-dir <NAME>` for custom basenames. Repeat the option to add more
names. Custom names also work without the built-in preset:

```sh
lsp --tree --prune-dir target --prune-dir dist project
```

Pruning only controls recursive descent. It does not hide matching directories
from their parent listing, and it does not apply to explicit directory
operands.

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

Set `LSP_CONFIG_FILE` to a non-empty path to override the platform-default
config file. An unset or empty value keeps the normal platform path.

Long-format output shows symbolic permissions by default. Use
`--permissions octal` to replace them with octal permission bits,
`--permissions both` to add octal bits after the symbolic field, or
`--permissions none` to omit permission fields.

On Windows, long format shows native file attributes for `symbolic` display.
`--attributes long` is the default and shows readable names.
`--attributes short` uses a fixed-position 17-character prefix in
`RHSATPCONEIVBXQGF` order. Residual unknown bits append an
`Unknown(0xXXXXXXXX)` suffix. The `X` position represents `EA`, including the
aliased `RecallOnOpen` bit, and `F` represents `RecallOnDataAccess`.
`--permissions none` omits the column; `octal` and `both` remain unsupported
with Windows long output.

The attribute setting has no effect on short-format output. It is accepted but
ignored on Linux and macOS, where permission rendering remains unchanged.

Use `--header` with long-format output to add a title row for the active
columns. It has no effect on short output. In the config file, set
`header = true` alongside `long_format = true` or `tree = true`.

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

Set default options in the configuration file.

![lsp output](./images/screenshot.png)

Add `-D` in native mode, or `--group-directories-first` in GNU mode, to sort
directories first:

![lsp output](./images/screenshot2.png)
