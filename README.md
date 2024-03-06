# cloudbit-software
Software for the littleBits cloudBit (which was deprecated) so that it can connect to a server.

*This version is intended for communicating with the main server.*
*If you are looking to set it up for a local server, see the `udp` branch. (NOT TESTED THOROUGHLY)*

## stats
*(NOTE: UPDATE + FURTHER TESTING NEEDED)*

Memory usage is around 0.59 MB.
CPU usage is always less than 5%.

## quick start
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

## protocol details
The WebSocket exchanges and expects JSON strings/buffers on the stream. JSON not following the schema below is logged and ignored.

The root *object* should always have an `opcode` key, whose value should be a number.

When the `opcode` is equal to `0x1` (INPUT) or `0x2` (OUTPUT), a `data` object with the property `value` (number) can be (for INPUT payloads)/is (for OUTPUT payloads) expected in the root.

Opcode `0x3` (IDENTIFY) is used right after the WebSocket handshake completes and the connection is established. IDENTIFY is sent from the client, but should never be sent from the server. An IDENTIFY payload has a `mac_address` (string) property and a `cb_id` (string) property.

# license
cloudbit-software © 2024 by littleBitsman is licensed under CC BY-NC-SA 4.0. To view a copy of this license, visit http://creativecommons.org/licenses/by-nc-sa/4.0/

## notes
huge thanks to [Hixie](http://github.com/Hixie) who made the [localbit](https://github.com/Hixie/localbit) repository which helped me program this
