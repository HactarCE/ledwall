#/bin/bash

BIN_NAME=ledwall_rpi

set -e
cross build --release --target armv7-unknown-linux-musleabihf --bin "$BIN_NAME"
echo "Build successful!"

echo "Killing old process '$BIN_NAME' ..."
set +e
ssh pi "sudo pkill '$BIN_NAME'"
set -e

echo "Copying new file ..."
scp target/armv7-unknown-linux-musleabihf/release/ledwall_rpi pi:~

echo "Running process '$BIN_NAME' ..."
ssh pi 'sudo nohup ~/'"$BIN_NAME"' >/dev/null 2>&1 </dev/null & disown'

echo "Done!"
