# TODO

- [ ] migrate docs from `mdbook` to `zensical`
- [ ] complete code coverage - 83 lines uncovered at last run
- [ ] add `zizmor` linting to the GitHub actions - both local and as an action
      itself. Use `https://github.com/GitHubSecurityLab/actions-permission` to
      help.
- [ ] add colorization for different file types, folders and symlinks. Make it
      customizable and theme-able. Make it default but allow an option to
      disable it (or vice-versa). Files that have a known extension should all
      be colored the same way, and different to unknown file tipes.
- [ ] Add icons for partials like `TODO.*`, `LICENSE.*` and more - brands like
  claude, codex, vscode and more where the nerdfont exists
- [ ] using the config file, allow extending the existing file and folder
      mapping, or deleting specific maps.
- [ ] When adding recursion or tree-style output, revisit whether directory
      traversal should move over to the `ignore` crate instead of the current
      custom walker.
- [ ] Unify recursive and tree traversal policy behind a shared walker so
      depth limits, symlink handling, pruning, and error handling cannot drift
      between output modes.
- [ ] Revisit recursive operand error semantics so explicit file-operand stat
      errors can be reported without aborting later directory walks.
- [ ] Avoid duplicate stderr for already-reported recursive traversal errors
      while still returning a non-zero exit status.
- [ ] Add configurable tree rendering styles, including the current compact
      root display, classic root branch graphics, and an ASCII fallback.
- [ ] Consider GNU-style `total` lines or another consistent empty-directory
      marker for long and tree output, rather than special-casing single-root
      tree output.
- [ ] Add inode and allocated block size support for long-format output.
- [ ] Add explicit long-format header modes, keeping plain `--header` as an
      alias for per-section headers. Suggested modes: `section` for every
      recursive section, `once` for the first long-format table only, and
      `none` to disable config-enabled headers for one invocation.
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
      5. Avoid per-entry `symlink_metadata` on Unix where `DirEntry::file_type`
         and entry-name visibility rules are sufficient, while preserving the
         Windows reparse and hidden-attribute classification path.
      6. Cache platform-native entry-name sort keys before sorting, so Windows
         avoids repeated UTF-16 allocation and ordinal comparisons.
      7. Carry `EntryClassification` from directory collection into `FileInfo`
         construction so Windows reparse tags are not queried twice per entry.
      8. Avoid `current_dir()` lookup for already-absolute Windows paths before
         calling `FindFirstFileW` for reparse classification.
      9. Reuse a single `GitignoreCache` across recursive and tree traversal
         so ancestor ignore files are not rediscovered for every directory.
      10. Revisit `prettytable` for long recursive output; a custom row
         formatter may be leaner for hot paths.
- [ ] better handle dotfiles?
- [ ] option to list dotfiles (and folders) before non-dotfiles
- [ ] Investigate an optional name-shortening mode for very long filenames
      that preserves extensions without changing the default wrap behavior.
- [ ] Consider separating config-loaded values from effective runtime params so
      merge behavior is more explicit than the current shared `Params` type.
- [ ] Detect terminal color capability once at startup and pass the effective
      color mode/level through render paths, instead of re-checking
      `colored_text` capability while building long-format tables. Refactor
      `lsplus` render tests to inject explicit color levels rather than
      mutating `TERM`, `NO_COLOR`, or `COLORTERM`; leave the env/terminal
      detection matrix to `colored_text`.
- [ ] Extend human-readable size units beyond petabytes so exabyte-scale values
      render as `E` instead of large `P` multiples.
- [ ] Review `src/lib.rs` and crate/module visibility. Keep the current
      out-of-source unit test layout, but reduce the accidental public library
      API for this app crate where modules/items do not need to be exported.
- [ ] Improve rustdoc/docstring coverage in a dedicated docs PR. Start by
      running `RUSTDOCFLAGS='-D missing-docs' cargo doc --no-deps`; current
      known gaps include the public `structs` module export and public
      `Icon` enum variants.
- [ ] Continue shifting tests toward behavior-focused checks at module seams
      (`app`, `settings`, `render`) instead of broad smoke-style coverage.
- [ ] Evaluate `rstest` for table-driven tests/fixtures and `serial_test` for
      tests that mutate process-global state such as color mode or environment
      variables.
- [ ] Consider deriving or implementing `Default` for `cli::Flags` so tests can
      use struct update syntax instead of hand-rolling every flag field. Keep
      the parsed default path behavior (`"."`) explicit in tests that need it.
