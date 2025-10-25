#/bin/bash

set -e
EMOJI_HEX=$(python3 -c "print(hex(ord('$1'))[2:])")
EMOJI_SVG_URL="https://images.emojiterra.com/google/noto-emoji/unicode-16.0/color/svg/$EMOJI_HEX.svg"
curl "$EMOJI_SVG_URL" | rsvg-convert -w 32 -h 32 > /tmp/emoji-32x32.png
cargo run --package preprocess_image -- /tmp/emoji-32x32.png "/tmp/$EMOJI_HEX.rgb"
echo "Generated image! Copying to raspi ..."
scp "/tmp/$EMOJI_HEX.rgb" "pi:~/$EMOJI_HEX.rgb"
