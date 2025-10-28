#!/bin/bash

BIN_NAME=ledwall
export CARGO_TARGET_DIR=~/ledwall_target_cache
export CARGO_TERM_COLOR=always
export CARGO_TERM_PROGRESS_WHEN=always
export CARGO_TERM_PROGRESS_WIDTH=20

cd /tmp/ledwall

echo "Killing old process ..."
sudo pkill $BIN_NAME

set -e

echo "Building on raspi ..."
~/.cargo/bin/cargo build -p $BIN_NAME --no-default-features --features gilrs,rpi-led-panel
echo "Build successful!"

echo "Moving new file ..."
mv ~/ledwall_target_cache/debug/$BIN_NAME ~

echo "Running process '$BIN_NAME' ..."
sudo nohup ~/$BIN_NAME >/dev/null 2>&1 </dev/null & disown
