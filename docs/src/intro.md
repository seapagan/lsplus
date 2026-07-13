# An `ls` replacement written in `Rust`


LSPlus is a functional Unix `ls` clone written in Rust. I built it as a Rust
learning project, so some code may still show beginner decisions.

![lsp output](./images/screenshot.png)

## Compatibility

LSPlus supports Linux, macOS, and Windows. Windows output uses native file
attributes and recognises junctions separately from symbolic links.

## Nerd Fonts

Install a Nerd Font in your terminal to display folder and file icons. The
[Nerd Fonts website](https://www.nerdfonts.com/) has a broad selection.

My personal favourite is `MesoLG Nerd Font`. Configure your terminal to use the
font after installing it.

Icons are displayed automatically when stdout is a terminal or a regular file,
and omitted from pipes. Use `--icons=always` to retain them through a
Unicode-aware pipe, or `--no-icons` to disable them completely.
