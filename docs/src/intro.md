# An `ls` replacement written in `Rust`


This is currently a very simple (though functional) clone of the Unix 'ls'
command written in Rust. It is a learning project for me to learn Rust so
probably contains many inefficiencies and bad practices. I'll get better 😁

![lsp output](./images/screenshot.png)

## Compatibility

This project is currently only compatible with Unix-like systems (Linux,
MacOs, etc.). Windows support is planned to be added very soon.

## Nerd Fonts

To display the folder and file icons, you need to first install a 'Nerd Font'
for your terminal. You can find a great selection of Nerd Fonts
[here](https://www.nerdfonts.com/)

My personal favourite is `MesoLG Nerd Font`, but there are many others to choose
from. You will also need to set up your terminal to use that font.

If you **DO NOT** want to install a Nerd Font, pass the `--no-icons` switch to
the program.
