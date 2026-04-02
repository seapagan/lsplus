# TODO

- [ ] add colorization for different file types, folders and symlinks. Make it
      customizable and theme-able. Make it default but allow an option to
      disable it (or vice-versa). Files that have a known extension should all
      be colored the same way, and different to unknown file tipes.
- [ ] for a symlink, color the name as it is, but color the target depending on
      whether it is a directory, file, or symlink.
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
- [ ] Consider separating config-loaded values from effective runtime params so
      merge behavior is more explicit than the current shared `Params` type.
- [ ] Review crate/module visibility and reduce the public surface where items
      do not need to be exported.
- [ ] Continue shifting tests toward behavior-focused checks at module seams
      (`app`, `settings`, `render`) instead of broad smoke-style coverage.
