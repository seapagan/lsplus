# Configuration File

Configure `lsplus` with a **`TOML`** file at:

- Linux: `~/.config/lsplus/config.toml`
- macOS: `~/.config/lsplus/config.toml`
- Windows: `%APPDATA%\\lsplus\\config.toml`

The configuration file is optional. `lsplus` uses default settings when the file
does not exist.

Set `LSP_CONFIG_FILE` to a non-empty path to use that file instead of the
platform default. An unset or empty value keeps the normal platform path.

`lsplus` also supports an `LSP_COMPAT_MODE` environment variable. When set, it
overrides the `compat_mode` value from the selected config file.

## Available Options

The configuration file supports these command-line options:

### compat_mode

- Permitted values: `"native"` or `"gnu"`
- Default value: `"native"`

This option selects which command-line interface `lsp` uses at startup.
`native` keeps the standard `lsplus` CLI, while `gnu` enables the GNU `ls`
compatibility surface intended for aliases and scripts.

At the moment, `gnu` mode changes the CLI surface and help output only. The
conflicting GNU short flags `-D`, `-I`, `-N`, and `-Z` are reserved in that
mode and will error until their GNU behavior is implemented.

### show_all

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-a` or `--all` and displays hidden files when set to
`true`.

### almost_all

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-A` or `--almost-all` and displays hidden files
except `.` and `..` when set to `true`.

### indicator_style

- Permitted values: `"none"`, `"slash"`, `"file-type"`, or `"classify"`
- Default value: `"none"`

This option selects which file type indicators `lsp` appends to entry names.
In native mode, the related CLI options are `-p` / `--slash-dirs`,
`--file-type`, `-F` / `--classify`, and `--no-indicators`. In `gnu`
compatibility mode, the equivalent GNU forms are `-p`,
`--indicator-style=slash`, `--file-type`,
`--indicator-style=file-type`, `-F`, `--indicator-style=classify`, and
`--indicator-style=none`.

The indicator characters are `/` for directories, `@` for symlinks, `|` for
FIFOs, `=` for sockets, and `*` for executables. The `*` executable marker is
only added by `"classify"`.

In long format, native mode omits the symlink `@` marker because `name ->
target` and the symlink styling already make the type clear. This also matches
GNU `ls`, which does not append `@` to symlink names in long format.

For backward compatibility, `append_slash = true` is still accepted in the
config file and maps to `indicator_style = "slash"`. If both are present,
`indicator_style` takes precedence.

### dirs_first

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `--sort-dirs` and sorts directories before files when
set to `true`. In `gnu` compatibility mode, the
equivalent long option is `--group-directories-first` (replacing the original
`--sort-dirs`).

### long_format

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `--long` and displays output in long format when set
to `true`.

### short_format

- Permitted values: `"vertical"` or `"across"`
- Default value: unset

Short output uses vertical columns when stdout is a terminal and one entry per
line when stdout is redirected. Set `short_format = "vertical"` to force the
vertical grid for redirected output, or set `short_format = "across"` to fill
rows from left to right. The settings correspond to `-C` /
`--format=vertical` and `-x` / `--format=across`; either explicit format keeps
the grid when output is redirected and has no effect on long or tree output.

### header

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `--header` and adds a title row to long-format
output when set to `true`. It only affects long-format output, so use it with
`long_format = true` or `tree = true`.

### human_readable

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-h` or `--human-readable` and displays
human-readable file sizes using powers of 1024 when set to `true`.

### si

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `--si` and displays human-readable file sizes using
powers of 1000 when set to `true`. It also enables human-readable size output.

### recursive

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-R` or `--recursive` and lists subdirectories in
separate `path:` sections.

### tree

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `--tree` and displays long-format tree output. Tree
output implies long format.

### tree_level

- Permitted values: integers greater than or equal to `1`
- Default value: `2`

This option corresponds to `--level` and limits recursive or tree output to
visible entry levels below each operand. A value of `1` shows only entries
directly under the requested directory; a value of `2` also shows
grandchildren. Without this option, recursive output is unlimited.

### prune_noisy_dirs

- Permitted values: `true` or `false`
- Default value: `false`

This option enables the built-in traversal prune preset for recursive and tree
output. Matching directories still appear in their parent listing, but `lsp`
does not descend into them. The preset matches these exact basenames:

- `.git`
- `.hg`
- `.svn`
- `node_modules`
- `__pycache__`

Pruning only applies while traversing children for `recursive = true` or
`tree = true`. It does not hide matching entries, does not apply to explicit
directory operands, and is not disabled by `show_all = true`.

### prune_dirs

- Permitted values: an array of strings
- Default value: `[]`

This option adds custom exact directory basenames to skip while traversing
recursive and tree output. Custom prune names apply even when
`prune_noisy_dirs = false`.

For example:

```toml
prune_dirs = ["target", "dist"]
```

### icons

- Permitted values: `"auto"`, `"always"`, or `"never"`
- Default value: `"auto"`

This option controls icons in short, long, recursive, and tree output. `auto`
shows icons when stdout is a terminal or regular file but omits them from
pipes, `always` retains them everywhere, and `never` disables them. It
corresponds to `--icons=auto|always|never`.

### no_icons

- Permitted values: `true` or `false`
- Default value: `false`

This compatibility option corresponds to `--no-icons` and is equivalent to
`icons = "never"` when set to `true`. An explicit `icons` value in the same
config file overrides `no_icons`; either CLI icon option overrides the config.

### no_color

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-N` or `--no-color` and disables colored and styled
output when set to `true`.

### permission_colors

- Permitted values: `true` or `false`
- Default value: `true`

This option controls long-format colors for the file type character and
permission bits. Set it to `false`, or pass `--no-permission-colors`, to render
those fields without accent colors.

### permissions

- Permitted values: `symbolic`, `octal`, `both`, or `none`
- Default value: `symbolic`

This option corresponds to `--permissions` and controls long-format permission
fields. `symbolic` shows the default file type character and symbolic
permissions, `octal` replaces that field with the file type character and
four-digit octal permission bits, `both` adds an octal permission cell after
the symbolic field, and `none` omits permission fields.

On Windows, `symbolic` shows a readable file-attribute column instead. `octal`
and `both` are unsupported when long format is active; use `symbolic` or
`none`.

### attributes

- Permitted values: `long`, `short`, or `minimal`
- Default value: `long`

This option corresponds to `--attributes` and controls text in the Windows
long-format `Attributes` column. `long` shows readable attribute names. `short`
shows a fixed-position 17-character prefix in `RHSATPCONEIVBXQGF` order, with
unset positions rendered as `-`. Residual unknown bits append an
`Unknown(0xXXXXXXXX)` suffix.

`minimal` shows the fixed four-character `RHSA` field and uses `Attr` as the
column header. Other known attributes are omitted, while genuinely unknown
bits still append the same suffix.

The `X` position represents `EA`, including the aliased `RecallOnOpen` bit, and
`F` represents `RecallOnDataAccess`. `permissions = "none"` omits the Windows
attribute column regardless of this setting. The setting has no effect on
short-format output and is accepted but ignored on Linux and macOS.
See [Windows attribute characters](usage.md#windows-attribute-characters) for
the complete short-field mapping.

### time_gradient

- Permitted values: `true` or `false`
- Default value: `true`

This option controls long-format timestamp freshness colors. Set it to `false`,
or pass `--no-time-gradient`, to use the fixed timestamp color instead of
age-based timestamp colors.

Timestamp colors adapt to terminal color capability: truecolor terminals use a
smooth age gradient and 256-color terminals use a stepped fallback to
distinguish day, week, month, and year bands. Basic ANSI terminals use named
yellow styling. Terminals without color support render timestamps plainly.
Future-dated timestamps stay red, even with `time_gradient` disabled, to make
clock-skewed files stand out.

### size_colors

- Permitted values: `true` or `false`
- Default value: `true`

This option controls long-format large-size colors. Set it to `false`, or pass
`--no-size-colors`, to render sizes without large-file accents.

### gitignore

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-I` or `--gitignore` and dims entries matched by the
active Git ignore rules, including merged `.gitignore` files,
`.git/info/exclude`, and the configured global Git excludes file.

### fuzzy_time

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-Z` or `--fuzzy-time` and displays timestamps in a
fuzzy format when set to `true`.

## Example Configuration File

This example sets several options. Omitted options use default values:

```toml
# compat_mode = "native"  # or "gnu" for GNU ls compatibility
show_all = true
indicator_style = "classify"
dirs_first = true
long_format = true
# short_format = "vertical"  # or "across"
# header = true
human_readable = true
# si = true
# recursive = true
# tree = true
# tree_level = 2
# prune_noisy_dirs = true
# prune_dirs = ["target", "dist"]
icons = "auto"
no_color = true
permission_colors = false
permissions = "symbolic"
attributes = "long"
time_gradient = false
size_colors = false
fuzzy_time = true
gitignore = true
```
