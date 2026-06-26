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
- [ ] better handle dotfiles?
- [ ] option to list dotfiles (and folders) before non-dotfiles
- [ ] Investigate an optional name-shortening mode for very long filenames
      that preserves extensions without changing the default wrap behavior.
- [ ] Consider colouring special permission bits (`s`, `S`, `t`, `T`) in
      long-format permission output, with tests for setuid, setgid, and sticky
      modes.
- [ ] Consider separating config-loaded values from effective runtime params so
      merge behavior is more explicit than the current shared `Params` type.
- [ ] Extend human-readable size units beyond petabytes so exabyte-scale values
      render as `E` instead of large `P` multiples.
- [ ] Review `src/lib.rs` and crate/module visibility. Keep the current
      out-of-source unit test layout, but reduce the accidental public library
      API for this app crate where modules/items do not need to be exported.
- [ ] Continue shifting tests toward behavior-focused checks at module seams
      (`app`, `settings`, `render`) instead of broad smoke-style coverage.
