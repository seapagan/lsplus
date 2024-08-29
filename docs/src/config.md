# Configuration File

It is possible to configure `lsplus` using a configuration file. The
configuration file is a simple **`TOML`** file that is placed in the following
location:

- Linux: `~/.config/lsplus/config.toml`
- MacOS: `~/.config/lsplus/config.toml`

The configuration file is optional and if it is not found, `lsplus` will use the
default settings.

## Available Options

The following options are available in the configuration file and correspond to
the relevant command line options:

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
will append a slash to directories if set to `true`.

### dirs_first

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `--sort-dirs` command line option and will

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

### fuzzy_time

- Permitted values: `true` or `false`
- Default value: `false`

This option corresponds to the `-Z` or `--fuzzy-time` command line option and
will display the time in a 'fuzzy' format if set to `true`.

## Example Configuration File

The following is an example configuration file that sets several options. Any
options that are not set will use the default values:

```toml
show_all = true
append_slash = true
dirs_first = true
human_readable = true
fuzzy_time = true
```
