# TODO

- [ ] add colorization for different file types, folders and symlinks. Make it
      customizable and theme-able. Make it default but allow an option to
      disable it (or vice-versa). Files that have a known extension should all
      be colored the same way, and different to unknown file tipes.
- [ ] Add icons for partials like `TODO.*`, `LICENSE.*` and more - brands like
  claude, codex, vscode and more where the nerdfont exists
- [ ] using the config file, allow extending the existing file and folder
      mapping, or deleting specific maps.
- [ ] add a -R flag to recursively list files in a directory.
- [ ] When adding recursion or tree-style output, revisit whether directory
      traversal should move over to the `ignore` crate instead of the current
      custom walker.
- [ ] Add inode, allocated block size, and optional column header support for
      long-format output.
- [ ] Evaluate the Rust crate `uutils-term-grid` as a short-format layout
      alternative before expanding the current custom grid code.
- [ ] Improve listing performance with focused architecture changes, in this
      order:
      1. Add a short-format entry model so short output does not build full
         long-format `FileInfo` data such as owner/group names, permissions,
         size, mtime, and long symlink target text.
      2. Pass a shared buffered stdout writer through render paths so recursive
         streaming does not pay for many small stdout writes.
      3. Cache UID-to-user and GID-to-group lookups during long-format runs.
      4. Carry cheap `DirEntry::file_type()` data through directory filtering
         and sorting so short mode can avoid extra metadata calls where
         possible.
      5. Revisit `prettytable` for long recursive output; a custom row
         formatter may be leaner for hot paths.
- [ ] better handle dotfiles?
- [ ] option to list dotfiles (and folders) before non-dotfiles
- [ ] Investigate an optional name-shortening mode for very long filenames
      that preserves extensions without changing the default wrap behavior.
- [ ] Consider separating config-loaded values from effective runtime params so
      merge behavior is more explicit than the current shared `Params` type.
- [ ] Extend human-readable size units beyond petabytes so exabyte-scale values
      render as `E` instead of large `P` multiples.
- [ ] Review `src/lib.rs` and crate/module visibility. Keep the current
      out-of-source unit test layout, but reduce the accidental public library
      API for this app crate where modules/items do not need to be exported.
- [ ] Continue shifting tests toward behavior-focused checks at module seams
      (`app`, `settings`, `render`) instead of broad smoke-style coverage.
