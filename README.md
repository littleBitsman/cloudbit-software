# cloudbit-software
Firmware for the littleBits CloudBit (which was deprecated) so that it can connect to a server.
*This version is intended for communicating with the main server.*
*If you are looking to set it up for a local server, see the `udp` branch. (NOT TESTED THOROUGHLY)*

## stats
*(NOTE: UPDATE + FURTHER TESTING NEEDED)*

Memory usage is around 0.59 MB.
CPU usage is always less than 5%.

# quick start
**You need any computer that is able to mount a ext2 or ext3 filesystem, like Linux or a Mac, for any steps following this. Windows has downloadable tools from the Internet to do this, but use them *at your own risk*.**
**I HEAVILY recommend saving an image of the current state of the drive, using tools like `dd`, before installing.**
1. mount your cloudBit SD card (the root of the mount will be referenced as `~`)
2. download the binary `cloud_client`
3. copy it into `~/usr/local/lb/cloud_client/bin`, renaming the already existing file if you wish to keep it as a backup or overwriting
4. done!

**If you want the binary to use a different server than the default, do the following:**
1. create a file `~/usr/local/lb/cloud_client/server_url`
2. put the FULL URL in the file, including `ws://` or `wss://` at the start - if the URL is invalid the default will automatically be used

**To build manually:**
1. clone the full GitHub repo from `main` (root of this directory will be referenced as `./`)
2. install the Rust tools for your OS if you don't already have them (`rustup`, `cargo`, etc.)
3. run `rustup target add armv5te-unknown-linux-musleabi`
4. run `cargo install cross`
5. run `cross build --release --target armv5te-unknown-linux-musleabi`
6. your binary will be found at `./target/armv5te-unknown-linux-musleabi/release/cloud_client`

# license
cloudbit-software © 2024 by littleBitsman is licensed under CC BY-NC-SA 4.0. To view a copy of this license, visit http://creativecommons.org/licenses/by-nc-sa/4.0/
