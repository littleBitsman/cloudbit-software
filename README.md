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
1. mount your cloudBit SD card (the root of the mount will be referenced as `~`)
2. download the binary `cloud_client`
3. copy it into `~/usr/local/lb/cloud_client/bin` (rename the already existing file if you wish to keep it as a backup)
4. create a file `~/usr/local/lb/cloud_client/local_server_url`
5. put the IP address and port of the server into the file; it should be with no newlines and look like `192.168.1.155:3000`
6. done!

**If you want the binary to use a different local port than the default (3000), do the following:**
1. create a file `~/usr/local/lb/cloud_client/local_port`
2. put any valid port number (1024-65535, anything lower than 1024 is usually restricted) - invalid numbers are ignored

**To build manually:**

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
 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|             MAC ADDRESS           |IO|VALUE|
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
                                     |    |
                                     |    |
I -> Input, O -> Output -------------'    |
Input/Output value -----------------------' - A 16-bit unsigned integer is expected for output, the input is always an unsigned 8-bit integer.
```

If bytes 12-14 do not exist, then the packet can be considered an `IDENTIFY` packet.

# license
cloudbit-software © 2024 by littleBitsman is licensed under CC BY-NC-SA 4.0. To view a copy of this license, visit http://creativecommons.org/licenses/by-nc-sa/4.0/

## notes
huge thanks to [Hixie](http://github.com/Hixie) who made the [localbit](https://github.com/Hixie/localbit) repository which helped me program this
