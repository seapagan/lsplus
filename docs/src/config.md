# Configuration File

Configure `lsplus` with a **`TOML`** file at:

- Linux: `~/.config/lsplus/config.toml`
- macOS: `~/.config/lsplus/config.toml`

The configuration file is optional. `lsplus` uses default settings when the file
does not exist.

`lsplus` also supports an `LSP_COMPAT_MODE` environment variable. When set, it
overrides the `compat_mode` value from the config file.

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

### human_readable

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `-h` or `--human-readable` and displays
human-readable file sizes when set to `true`.

### no_icons

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to `--no-icons` and hides icons when set to `true`.

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

### time_gradient

- Permitted values: `true` or `false`
- Default value: `true`

This option controls long-format timestamp freshness colors. Set it to `false`,
or pass `--no-time-gradient`, to use the fixed timestamp color instead of
age-based timestamp colors.

Timestamp colors adapt to terminal color capability: truecolor terminals use a
smooth age gradient and 256-color terminals use a stepped fallback to
distinguish day, week, month, and year bands. Basic ANSI terminals use named
yellow styling. Future-dated timestamps stay red, even with `time_gradient`
disabled, to make clock-skewed files stand out.

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
human_readable = true
no_color = true
permission_colors = false
time_gradient = false
size_colors = false
fuzzy_time = true
gitignore = true
```
