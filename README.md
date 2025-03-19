# walltaker-rust
Another Walltaker client, written in Rust for Linux.

I know this isn't good, but I made this for myself. It uses more-wallpapers, allowing multi-monitor support.
The program automatically creates a links.txt file in your working directory, edit the file to have your Walltaker links.

To build, change the fallback wallpaper in src/main.rs, then you should be able to just `cargo build` it, I hope.

If you want to, you can also set the wallpaper to download somewhere outside /tmp/, and the fallback doesn't matter.

(there's 0 error checking, and i've never written rust before. good luck)
