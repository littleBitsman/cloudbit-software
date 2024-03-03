# pull
git pull

# build
cross build --release --target armv5te-unknown-linux-musleabi
mv target/armv5te-unknown-linux-musleabi/release/cloud_client ./

# commit
git commit -a -m "build"
git push