# walltaker-rust
Another Walltaker client, written in Rust for Linux.

The reason I wrote this is to give myself multi-monitor support on linux. For most people, simpler solutions will be better. This also doesn't work on Windows.

When ran for the first time, the program will ask for configuration. You should be able to follow the on screen instructions. If there's a config error, run the program as `./walltaker-rust --fix` to fix errors. The program only asks for user input on first run and when ran with `--fix`.

Searches for config at $XDG_CONFIG_HOME/walltaker.toml, then $HOME/.config/walltaker.toml.

I've only tested KDE, but Sway, XFCE, and Cinnamon should work. The full list of supported DEs is [here](https://crates.io/crates/more-wallpapers).

To build: (On Debian, if you use arch and you can't figure this out i can't help you)
```
sudo apt install cargo libssl libsrandr-dev
git clone https://github.com/donordirt/walltaker-rust.git
cd walltaker-rust
cargo build --release
```
