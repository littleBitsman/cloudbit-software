cargo build --release --target armv5te-unknown-linux-musleabi --static
mv target/armv5te-unknown-linux-musleabi/release/cloud_client ./

# commit
git commit -a -m "build"
git push