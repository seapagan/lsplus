# Configuration File

It is possible to configure `lsplus` using a configuration file. The
configuration file is a simple **`TOML`** file that is placed in the following
location:

- Linux: `~/.config/lsplus/config.toml`
- MacOS: `~/.config/lsplus/config.toml`

The configuration file is optional and if it is not found, `lsplus` will use the
default settings.

`lsplus` also supports an `LSP_COMPAT_MODE` environment variable. When set, it
overrides the `compat_mode` value from the config file.

## Available Options

The following options are available in the configuration file and correspond to
the relevant command line options:

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

This option corresponds to the `-a` or `--all` command line option and will
display all files and directories if set to `true`, including hidden files.

### almost_all

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `-A` or `--almost-all` command line option and
will display all files and directories if set to `true`, except for `.` and
`..`.

### append_slash

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `-p` or `--slash-dirs` command line option and
will append a slash to directories if set to `true`. In `gnu` compatibility
mode, the equivalent long option is `--indicator-style=slash`.

### dirs_first

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `--sort-dirs` command line option and will sort
directories before files when set to `true`. In `gnu` compatibility mode, the
equivalent long option is `--group-directories-first` (replacing the original
`--sort-dirs`).

### long_format

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `--long` command line option and will display the
output in long format if set to `true`.

### human_readable

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `-h` or `--human-readable` command line option
and will display file sizes in human readable format if set to `true`.

### no_icons

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `--no-icons` command line option and will not
display icons if set to `true`.

### no_color

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `-N` or `--no-color` command line option and
will disable colored and styled output if set to `true`.

### gitignore

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `-I` or `--gitignore` command line option and
will dim entries that are matched by the active Git ignore rules, including
merged `.gitignore` files, `.git/info/exclude`, and the configured global Git
excludes file.

### fuzzy_time

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `-Z` or `--fuzzy-time` command line option and
will display the time in a 'fuzzy' format if set to `true`.

## Example Configuration File

The following is an example configuration file that sets several options. Any
options that are not set will use the default values:

```toml
compat_mode = "gnu"
show_all = true
append_slash = true
dirs_first = true
human_readable = true
no_color = true
fuzzy_time = true
gitignore = true
```
