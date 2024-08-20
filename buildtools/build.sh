# This script is used for my QoL during programming since it is insanely annoying to type/do it manually
# Purpose: Pull any changes, build binary for the respective ARM version and Linux distro, copy it to the root of the repo, and push to GitHub
# THIS SCRIPT IS NOT MEANT TO BE KEPT

# pull
git pull

# build
# apparently gnueabi uses local system deps., and musleabi doesn't so I'm using that now :)
cross build --release --target armv5te-unknown-linux-musleabi