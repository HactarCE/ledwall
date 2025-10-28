#!/bin/bash

./reprocess_images.sh

BIN_NAME=ledwall

set -e

echo "Copying files to raspi ..."
rsync --exclude .git --exclude target --exclude img/png --recursive --delete --compress --times . pi:/tmp/ledwall

ssh pi "/tmp/ledwall/run_on_pi.sh"

echo "Done!"
