#!/bin/bash

[ -d img/png ] || exit

/bin/rm -rf img/rgba
cargo run -p preprocess_image -- img/png img/rgba
echo "Done!"
