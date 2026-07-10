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

If you **DO NOT** want to install a Nerd Font, pass the `--no-icons` switch to
the program.
