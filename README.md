# ledwall

32x64 RGB LED wall that runs on a Raspberry Pi

## Issues

### Tetris

- DAS on move
- no repeat on hard drop and rotate
- fall
- lock down
- review mappings
- more rendering

## Testing

```sh
cargo run --bin demo --features macroquad
```

## Deployment

1. Install Docker (for `cross`)
2. `cargo install cross --git https://github.com/cross-rs/cross`
3. `docker pull --platform linux/amd64 ghcr.io/cross-rs/x86_64-unknown-linux-musl:0.2.5`

```sh
rustup target add armv7-unknown-linux-gnueabihf
cargo build --target armv7-unknown-linux-gnueabihf --bin ledwall --features rpi-led-matrix
```
