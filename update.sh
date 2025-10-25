#/bin/bash

BIN_NAME=ledwall

echo "Killing old process '$BIN_NAME' ..."
ssh pi "sudo pkill $BIN_NAME"

set -e

echo "Copying files to raspi ..."
rsync --exclude .git --exclude target --recursive --delete . pi:/tmp/ledwall
echo "Building on raspi ..."
ssh pi "cd /tmp/ledwall && CARGO_TARGET_DIR=~/ledwall_target_cache CARGO_TERM_COLOR=always CARGO_TERM_PROGRESS_WHEN=always CARGO_TERM_PROGRESS_WIDTH=20 ~/.cargo/bin/cargo build -p $BIN_NAME --no-default-features --features gilrs,rpi-led-panel"
echo "Build successful!"

# cross build --release --target armv7-unknown-linux-musleabihf --bin "$BIN_NAME"
# echo "Build successful!"

echo "Copying new file ..."
ssh pi "cp ~/ledwall_target_cache/debug/$BIN_NAME ~"
# scp target/armv7-unknown-linux-musleabihf/release/"$BIN_NAME" pi:~

echo "Running process '$BIN_NAME' ..."
ssh pi "sudo nohup ~/$BIN_NAME >/dev/null 2>&1 </dev/null & disown"

echo "Done!"
