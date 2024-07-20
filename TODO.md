# TODO

- [ ] in short mode when `-p` is specified, show a '*' or something if the file
is a symlink.
- [ ] add option `-h` to show file sizes in 'human readable' format.
- [ ] only show hidden files if `-a` is specified. At the moment, hidden files
are always shown.
- [x] sorting to put folders first, then files, then symlinks.
- [ ] add more icons for different file types. Implement a module that simply
maps icons to file-types, and file file-types to extensions.
- [ ] add colorization for different file types, folders and symlinks. Make it
customizable and theme-able. Make it default but allow an option to disable it.
(or vice-versa).
- [ ] add configuration file to set options. Use TOML format by default,
maybe adding YAML later. Store in the correct `.config` directory depending on
the OS.
- [ ] for a symlink, color the name as it is, but color the target depending on
whether it is a directory, file, or symlink.
