# Russs

Russs is a toy implementation of [Shamir's Secret Sharing](https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing), based on [Hiss](https://github.com/Tak/hiss).

*DO NOT* use it for anything critical: it's based on a cryptographically secure algorithm, but my implementation may have inadvertently introduced a weakness.

However, feel free to have fun with it!

![Generating keys from a file](https://raw.githubusercontent.com/Tak/hiss/master/images/generate.png)
![Reconstructing a file](https://raw.githubusercontent.com/Tak/hiss/master/images/reconstruct.png)

## Installation
### From source
- Clone this repository
- [Install rust](https://www.rust-lang.org/tools/install)
- [Install Gtk+](https://www.gtk.org/download/index.php)
- Run `cargo run --release` from the `russs` directory

