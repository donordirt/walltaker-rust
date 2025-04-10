# walltaker-rust
Another Walltaker client, written in Rust for Linux.

The reason I wrote this is to give myself multi-monitor support. For most people, simpler solutions will be better. This also probably doesn't work on Windows.

When ran for the first time, the program will ask for configuration. You should be able to follow the on screen instructions.

Searches for config at $XDG_CONFIG_HOME/walltaker.toml, then $HOME/.config/walltaker.toml.

You can check desktop enviroment support is at https://crates.io/crates/more-wallpapers. I've only tested KDE, but everything else with per-screen support should work.

To build: (On Debian, if you use arch and you can't figure this out i can't help you)
`sudo apt install cargo libssl libsrandr-dev`
`git clone https://github.com/donordirt/walltaker-rust.git`
`cd walltaker-rust`
`cargo build --release`