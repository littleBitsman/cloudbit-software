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

*note that all steps are automatically handled by the auto installer, after using it there is no further action required.*

### manual build (for those who know what they are doing)
*This build method has only been tested on Linux. I recommend using a GitHub Codespace with a fork of this repo if you don't have access to a Linux computer.*
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

When the `opcode` is equal to `0x1` (INPUT) or `0x2` (OUTPUT), a `data` object with the property `value` (number).
The `data` object must exist for OUTPUT packets, and the `data` object is **guaranteed** to exist for INPUT packets. INPUT should never be sent by the server, and OUTPUT will never be sent by the client.

An INPUT or OUTPUT packet could look like this (note that `0x1` or `0x2` are not what the opcode value(s) would look like in JSON):
```js
// INPUT
{
    "opcode": 0x1,
    "data": {
        "value": 0
    }
}
// OUTPUT
{
    "opcode": 0x2,
    "data": {
        "value": 0
    }
}
```

Opcode `0x3` (IDENTIFY) is used right after the WebSocket handshake completes and the connection is established. IDENTIFY is sent from the client and should never be sent from the server. An IDENTIFY payload has a `mac_address` (string) property and a `cb_id` (string) property. 

An IDENTIFY packet could look like this (note that `0x3` is not what the opcode value would look like in JSON):
```js
{
    "opcode": 0x3,
    "mac_address": "00:00:00:00:00:00",
    "cb_id": "some_hash_thing"
}
```

### developer opcodes
These are opcodes that are available for use for any devs wanting to customize their cloudBits.

- `0xF0` (LED) is used if you ever want to tell the cloudBit to change the LED color at any time. A `commands` property (string) is expected. It can be any combination of `red`, `green`, `blue`, `yellow`, `teal`, `purple` (or `violet`), `white`, `off`, `blink` and `clownbarf`, with whitespace separating each command.
- `0xF1` (Button) requests that the cloudBit sends its current button status (true = pressed, false = not pressed). No fields are required other than the opcode itself. *Remember that when the button is pressed **and held** the cloudBit will enter commissioning mode and will disconnect from the server.*
    - `0xF2` is the return opcode (contains the button status)
        - An example button return opcode *could* look like this (note that `0xF2` is not what the opcode would look like in JSON)
        ```js
        {
            "opcode": 0xF2,
            "data": {
                "button": true
            }
        }
        ```
- `0xF3` requests that the cloudBit sends its current system stats (currently sends CPU usage as a percent, memory usage as a percent and in bytes, total memory in the system in bytes, and CPU die temperature in degrees Celsius). No fields are required other than the opcode itself.
    - `0xF4` is the return opcode (contains the statistics)
        - An example packet *could* look like this (note that `0xF4` is not what the opcode would look like in JSON)
        ```js
        {
            "opcode": 0xF4,
            "stats": {
                "cpu_usage": 10.6,
                "memory_usage": 5776,
                "memory_usage_percent": 10,
                "total_memory": 57760,
                "cpu_temp": 30
            }
        }
        ```
    - See the [Rust sysinfo crate](https://crates.io/crates/sysinfo) for more info on how system stats are retrieved
    - **WARNING: DO NOT POLL SYSTEM STATISTICS**

# versions
- `main` branch - version built every time a file in the src directory is updated - may be unstable
- `udp` branch - version built every time a file in the src directory on the `udp` branch is updated - may be unstable
- releases - versions ready to be used

# license
cloudbit-software - an alternative software for the littleBits cloudBit.

Copyright (C) 2024 littleBitsman

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see https://www.gnu.org/licenses/.

## notes
huge thanks to [Hixie](http://github.com/Hixie) who made the [localbit](https://github.com/Hixie/localbit) repository which helped me program this

reverse engineering notes can be found [here](https://github.com/littleBitsman/cloudbit-software/blob/main/reverse_engineering.md)
