cargo build --release --target armv5te-unknown-linux-gnueabi
mv target/armv5te-unknown-linux-gnueabi/release/cloud_client ./

# commit
git commit -a -m "build"
git push