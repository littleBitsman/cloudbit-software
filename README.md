# cloudbit-software
Software for the littleBits cloudBit (which was deprecated) so that it can connect to a server.

*This version is intended to communicate with a local (in your home network) server.* 
*If you are looking to use the main server, check the `main` branch.*

## stats
This version has not been tested yet.
Expected usage is as follows:
- CPU: < 5%
- Memory: < 0.5MB

## quick start
**You need any computer that is able to mount a ext2 or ext3 filesystem, like Linux or a Mac, for any steps following this. Windows has downloadable tools from the Internet to do this, but use them *at your own risk*.**
**I HEAVILY recommend saving an image of the current state of the drive, using tools like `dd`, before installing.**
### recommended method: auto installer
The auto installer can be found [here](https://github.com/littleBitsman/cloudbit-builder). If you do not wish to download arbitrary executables then you can use the alternate method (prebuilt binary) or manual build below. Instructions for the auto installer can be found in the README of its repository.

### alternate method: prebuilt binary
1. mount your cloudBit SD card (the root of the mount will be referenced as `~`)
2. download the binary `cloud_client`
3. copy it into `~/usr/local/lb/cloud_client/bin` (rename the already existing file if you wish to keep it as a backup)
4. at `~/usr/local/lb/cloud_client`, create a file named `udp_server_url` and place the URL of the UDP server you want the client to connect to WITHOUT ANY HTTP SCHEMES. it should look like a raw IP address/domain name and port (like `192.168.1.20:3000`)
5. done!
*note that all steps are automatically handled by the auto installer, after using it there is no further action required.*

### manual build (for those who know what they are doing)
*This build method has only been tested on Linux. I recommend using a GitHub Codespace with a fork of this repo if you don't have access to a Linux computer.*
1. install the Rust tools for your OS if you don't already have them (`rustup`, `cargo`, etc.)
2. clone the full GitHub repo from `udp` (root of this directory will be referenced as `./`)
4. traverse into the root directory of the clone
5. run `rustup target add armv5te-unknown-linux-musleabi`
6. run `cargo install cross`
7. run `cross build --release --target armv5te-unknown-linux-musleabi`
8. your binary will be found at `./target/armv5te-unknown-linux-musleabi/release/cloud_client`

## protocol details
UDP is a connectionless protocol; as such managing each cloudBit on your own server implementation may be difficult.

Each message in both directions ALWAYS has the first 12 bytes (UTF-8, for ease of development) being the respective letters in the MAC address (as in, characters 0 and 1 map to the first component of the MAC address). *In the future the client will parse the MAC address into the actual 6 bytes a MAC address has automatically, but that is currently WIP.*

The bytes are as follows:
```
 00 01 02 03 04 05 06 07 08
+--+--+--+--+--+--+--+--+--+
|    MAC ADDRESS  |IO|VALUE|
+--+--+--+--+--+--+--+--+--+
                    |    |
I -> Input, O -> Output  |
Input/Output value ------'
Value is always a 16-bit Little Endian unsigned integer, 0-255 for input and 0-65535 (or 0xFFFF) for output
```

If bytes 06-08 do not exist, then the packet can be considered an `IDENTIFY` packet.

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
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

## notes
huge thanks to [Hixie](http://github.com/Hixie) who made the [localbit](https://github.com/Hixie/localbit) repository which helped me program this
