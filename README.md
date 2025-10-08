# ledwall

32x64 RGB LED wall that runs on a Raspberry Pi

## Testing

```sh
cargo run --bin demo --features macroquad
```

## Deployment

```sh
rustup target add armv7-unknown-linux-gnueabihf
cargo build --target armv7-unknown-linux-gnueabihf --bin ledwall --features rpi-led-matrix
```
