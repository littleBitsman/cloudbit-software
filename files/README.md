These are files that were changed during reverse engineering (and also any changes that may be necessary for this software to work).

Any of these with a `**` next to them are necessary to be replaced. 
Any required changes as denoted below will be handled by [the cloudbit-builder installer](https://github.com/littleBitsman/cloudbit-builder) automatically.

Make sure you "save state" of your drive before doing any modifications on your own in case something happens.

Consider the root directory of the cloudBit SD card `~` for all of these.

- `onBoot.sh`: `~/usr/local/lb/bit-util/onBoot.sh` **
    - Changes made: modifying LED colors to make it more clear when something happens during boot
- `cloudclient.service`: `~/usr/lib/systemd/system` **
    - Changes made: adding a cooldown, adding LED colors after error and before startup, enabling logging to journal and stdout/stderr

### DO NOT TRY TO EXECUTE THESE FILES
*(I mean, you can, but why?)*
*((They wont work))*