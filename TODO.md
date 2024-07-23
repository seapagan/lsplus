# TODO

- [ ] in short mode when `-p` is specified, show a '*' or something if the file
is a symlink.
- [ ] add colorization for different file types, folders and symlinks. Make it
customizable and theme-able. Make it default but allow an option to disable it.
(or vice-versa). Files that have a known extension should all be colored the same
way, and different to unknown file tipes.
- [ ] add configuration file to set options. Use TOML format by default,
maybe adding YAML later. Store in the correct `.config` directory depending on
the OS.
- [ ] for a symlink, color the name as it is, but color the target depending on
whether it is a directory, file, or symlink.
- [ ] show executable files as bold.
- [ ] add option for 'fuzzy' dates, e.g. 'yesterday', 'last week', 'last month',
'last year', '2 years ago', etc.
- [ ] colorize the short-form output same as the long-form output.
- [ ] make file/folder icons optional and default to off.
- [ ] note in README that a `Nerd Font` is required for the icons to display
