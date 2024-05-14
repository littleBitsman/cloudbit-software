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
### recommended method: auto installer
The auto installer can be found [here](https://github.com/littleBitsman/cloudbit-builder). If you do not wish to download arbitrary executables then you can use the alternate method (prebuilt binary) or manual build below. Instructions for the auto installer can be found in the README of its repository.

### alternate method: prebuilt binary
1. mount your cloudBit SD card (the root of the mount will be referenced as `~`)
2. download the binary `cloud_client`
3. copy it into `~/usr/local/lb/cloud_client/bin` (rename the already existing file if you wish to keep it as a backup)
4. done!

**If you want the binary to use a different server than the default, do the following:**
1. create a file `~/usr/local/lb/cloud_client/server_url`
2. put the FULL URL in the file, including `ws://` or `wss://` at the start - if the URL is invalid the default will automatically be used

### manual build (not recommended)
1. install the Rust tools for your OS if you don't already have them (`rustup`, `cargo`, etc.)
2. clone the full GitHub repo from `main` (root of this directory will be referenced as `./`)
3. traverse into the root directory of the clone
4. run `rustup target add armv5te-unknown-linux-musleabi`
5. run `cargo install cross`
6. run `cross build --release --target armv5te-unknown-linux-musleabi`
7. your binary will be found at `./target/armv5te-unknown-linux-musleabi/release/cloud_client`

## protocol details
The opening HTTP request has `MAC-Address` and `CloudBit-Id` headers. The `MAC-Address` is the cloudBit's MAC address, and the `CloudBit-Id` is some hash of the MAC address. The main server uses these headers to authenticate the request. *In your own implementation for your personal use, you should have a list of MAC addresses, IDs, and their respective mappings.*

The WebSocket exchanges and expects JSON strings/buffers on the stream. JSON not following the schema below is logged and ignored.

The root *object* should always have an `opcode` key, whose value should be a number.

When the `opcode` is equal to `0x1` (INPUT) or `0x2` (OUTPUT), a `data` object with the property `value` (number) can be (for INPUT payloads)/is (for OUTPUT payloads) expected in the root.

Opcode `0x3` (IDENTIFY) is used right after the WebSocket handshake completes and the connection is established. IDENTIFY is sent from the client, but should never be sent from the server. An IDENTIFY payload has a `mac_address` (string) property and a `cb_id` (string) property.

Opcode `0x4` (LED) is used if you ever want to tell the cloudBit to change the LED color at any time. A `commands` property (string) is expected. It can be any combination of `red`, `green`, `blue`, `yellow`, `teal`, `purple` (or `violet`), `white`, `off`, `blink` and `clownbarf`, with whitespace separating each command (newlines are removed).

# license
cloudbit-software © 2024 by littleBitsman is licensed under CC BY-NC-SA 4.0. To view a copy of this license, visit http://creativecommons.org/licenses/by-nc-sa/4.0/

## notes
huge thanks to [Hixie](http://github.com/Hixie) who made the [localbit](https://github.com/Hixie/localbit) repository which helped me program this
