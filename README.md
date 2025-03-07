# mini-fe2io
A miniaturized version of FE2.IO written in Rust, and independent of any web browsers.
This is perfect for low end machines that can't handle playing games while having a browser open.

HUGE thanks to [@some100](https://github.com/some100) for reworking the entire code to make this work after I abandoned the project for 2 years.

# Building from Source

## Debian / Ubuntu: Install additional dependencies

```shell
sudo apt update
sudo apt install libasound2-dev libx11-dev
```

## Install Rust Toolchain
https://www.rust-lang.org/learn/get-started

## Clone the Repository:

```shell
git clone https://github.com/richardios275/mini-fe2io.git
cd mini-fe2io
```

## Build with Cargo
### Release builds
```shell
cargo build --release
```

### Debug builds
```shell
cargo build
```

## Run with Cargo
### Release builds
```shell
cargo run --release
```

### Debug builds
```shell
cargo run
```

# To-Do
These are several things I would like to do to make this more usable.
- Ability to switch audio device when default is disconnected / switched
- Audio Caching (Temp File)
- Timer to sync music when downloading

This page is a Work in Progress!
