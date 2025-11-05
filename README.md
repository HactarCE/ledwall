# ledwall

32x64 RGB LED wall that runs on a Raspberry Pi

## Activities

The LED wall supports several activities. Pressing the heart button (or "home" on most controllers) opens a menu that allows adjusting volume (currently unimplemented) and brightness and switching between activities.

### Life

_[Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) on a 32x64 torus_

By default the simulation runs at 40 FPS.

- Press X to clear the grid.
- Press Y to randomize the grid.
- Hold L to slow down to 12 FPS.
- Hold R to speed up to 120 FPS.

The simulation automatically re-randomizes a few seconds after it detects a cycle.

Planned features:

- [ ] Pausing
- [ ] Rewinding
- [ ] Editing individual cells
- [ ] Saving & loading patterns

### Tetris

_[Guideline](https://tetris.wiki/Tetris_Guideline)-compliant Tetris implementation_

The `tetris_logic` crate is completely frontend-independent and is designed to be reusable for other projects. Completed/planned features include:

- [x] Customizable game parameters via [`Config` struct](https://github.com/HactarCE/ledwall/blob/main/crates/tetris_logic/src/config.rs)
- [x] [SRS](https://tetris.wiki/Super_Rotation_System), including 180-degree rotation
- [x] [DAS](https://tetris.wiki/DAS)
- [x] Line clearing
- [x] Queue of next pieces (7-bag)
- [x] Hold piece
- [x] Animations on hard drop, lock, and line clear (implemented in frontend)
- [ ] Scoring
- [ ] [Gravity](https://tetris.wiki/Drop#Gravity)

## Testing locally

1. Install [Rust](https://rust-lang.org/tools/install/)
2. Run `reprocess_images.sh`, which converts PNG images to simple binary RGBA files
3. `cargo run`

Repeat step 2 whenever modifying images in `img/png`.

You can connect a controller for testing or use the keyboard with the following keybinds:

- <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>D</kbd> → directional pad
- <kbd>K</kbd> → A
- <kbd>J</kbd> → B
- <kbd>L</kbd> → X
- <kbd>I</kbd> → Y
- <kbd>X</kbd> → L (left bumper)
- <kbd>C</kbd> → L2 (left trigger)
- <kbd>,</kbd> → R2 (right trigger)
- <kbd>.</kbd> or <kbd>;</kbd> → R (right bumper)
- <kbd>return ↵</kbd> or <kbd>=</kbd> → + (plus)
- <kbd>-</kbd> → - (minus)
- <kbd>8</kbd> → ★ (star)
- <kbd>esc ⎋</kbd> → ❤ (heart/home)

You can customize these in `crates/ledwall/src/frontend_macroquad.rs`.

## Supplies

- [64x32 RGB LED matrix](https://www.adafruit.com/product/2276), which includes power and ribbon cables (I used 6mm pitch)
- [Adafruit RGB matrix bonnet for Raspberry Pi](https://www.adafruit.com/product/3211)
- [5V 4A switching power supply](https://www.adafruit.com/product/1466)
- Any Raspberry Pi with Wi-Fi (for SSH) and Bluetooth (for controllers). I used a [Raspberry Pi 3 Model B+](https://www.adafruit.com/product/3775) that I already had.
- Micro USB cable
- 2x [8BitDo Micro Bluetooth Gamepad](https://shop.8bitdo.com/products/8bitdo-micro-bluetooth-gamepad) (1 blue + 1 green), or any other two bluetooth controllers. I recommend dedicating controllers just for this project so you aren't fiddling with unpairing & repairing them.
- [Wall charger block with outlet + USB port](https://www.amazon.com/dp/B0F4CY6RDZ) (optional)

The total cost is ~$200, although you may already have some of the parts.

- ~$100 for the LED panel and everything needed to drive it
- ~$35-60 for the Raspberry Pi depending on model
- ~$50 for the controllers

You'll also need ability to solder and access to a soldering iron, solder, and a single very short jumper wire. You can probably find this at a local makerspace.

## Building

I followed the [Adafruit RGB Matrix Bonnet for Raspberry Pi](https://learn.adafruit.com/adafruit-rgb-matrix-bonnet-for-raspberry-pi/) tutorial.

1. Install a minimal OS onto the Raspberry Pi. I think I used Raspberry Pi OS Lite (64-bit).
2. Connect the Raspberry Pi to Wi-Fi. I don't remember how I did this. Good luck.
3. Set up SSH access to the Raspberry Pi. You can find guides for this online.
4. Follow the Adafruit guide to put the RGB matrix bonnet on the Raspberry Pi.
5. Test that you hooked everything up correctly by running the Adafruit demo on the Raspberry Pi:
    1. `sudo nano /boot/firmware/cmdline.txt` and add `isolcpus=3` to the end (with a space separating it from the rest of the line)
    2. `git clone https://github.com/hzeller/rpi-rgb-led-matrix`
    3. `cd rpi-rgb-led-matrix`
    4. `make`
    5. `sudo ./examples-api-use/demo -D 0 --led-rows 32 --led-cols 64 --led-gpio-mapping=adafruit-hat-pwm`
    6. Observe spinning square and rejoice.
6. Solder a very short jumper wire between pins 4 (OE) and 18 on the matrix bonnet.
7. Test that you soldered everything correctly by running the Adafruit demo in hardware pin-pulser mode:
    1. Disable the sound module[^sound]: `echo 'blacklist snd_bcm2835' | sudo tee /etc/modprobe.d/blacklist-rgb-matrix.conf`
    2. Reboot: `sudo reboot`
    3. `cd rpi-rgb-led-matrix`
    4. `sudo ./examples-api-use/demo -D 0 --led-rows 32 --led-cols 64 --led-gpio-mapping=adafruit-hat` (note no more `-pwm`)
    5. Observe spinning square and rejoice.

[^sound]: Something with the hardware pin pulser requires the sound module to be disabled, which prevents the 3.5mm audio jack and HDMI audio from working. You can still connect a [USB speaker](https://www.adafruit.com/product/3369) though, and I might make use of that in the future.

## Connecting Bluetooth controllers

For [testing locally](#testing-locally), I recommend pairing one controller to the LED wall itself and one controller to your development machine. Alternatively, you can use the keyboard controls on the development machine.

1. `sudo nano /etc/bluetooth/input.conf` and set `ClassicBondedOnly=false` as described in comment #6 of [this bug report](https://bugs.launchpad.net/ubuntu/+source/blueman/+bug/2046084). We don't care about security and I don't know how else to make this work.
2. `sudo systemctl restart bluetooth`
3. `sudo bluetoothctl` (future commands are inside the `bluetoothctl` prompt)
4. `scan on`
5. Hold the heart button on one of the controllers until its light starts blinking blue. You may need to charge the controller for a while first.
6. Wait until the controller appears in `bluetoothctl`. It should say `8BitDo Micro gamepad`. Note its MAC address, which will look something like `23:45:67:89:AB:CD`. (Write it down! You'll need it later.) Use that in the next several commands.
7. `pair 23:45:67:89:AB:CD`
8. `trust 23:45:67:89:AB:CD`
9. `connect 23:45:67:89:AB:CD`
10. Repeat steps 5-9 with the other controller.
11. `scan off`
12. `exit`

The controllers should now connect automatically whenever the Raspberry Pi and the controller are both on.

## Customizing and deploying the code

1. Install [Rust](https://rust-lang.org/tools/install/) on your development machine.
2. Create `~/.ssh/config` on your development machine if it doesn't yet exist and add the following lines, using your Raspberry Pi's local IP address and user account name:

```
Host pi
  HostName 192.168.xxx.yyy
  User whatever_your_username_is_on_the_raspberry_pi
```

3. Install [Rust](https://rust-lang.org/tools/install/) on the Raspberry Pi.
4. Run `./update.sh` on your development machine.
5. SSH into the Raspberry Pi and restart the LED wall: `sudo pkill ledwall && sudo ~/ledwall`.
6. Turn on the controllers and observe the UUIDs. If they are already on, turn them off and then on again while the program is running.
7. Edit `crates/ledwall/src/shell.rs` and set the `BLUE_CONTROLLER_UUID` and `GREEN_CONTROLLER_UUID` variables accordingly.

To update, simply run `./update.sh` on your development machine. This script assumes that you have an SSH host called `pi` to deploy to. It will automatically run `./reprocess_images.sh`.

If you are running Linux, you may be able to cross-compile instead of compiling the Rust code on the Raspberry Pi. I think this requires `pkg-config` being able to find headers for `uinput`, which I don't know how to do on macOS.
