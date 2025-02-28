# Torrenter

Torrenter is a lightweight and feature-rich BitTorrent client. It features a
clutter-free and modern GUI that is consistent across all platforms.

# Installation

As of now, the only way to install Torrenter is to build from source. We are
working on releasing pre-built distributions for Linux, macOS, and Windows. A
Cargo package is also in progress.

## Building with Cargo

Ensure that you have the following tools/libraries installed before starting the
build process:
- **Rust**
- **Clang** or **GCC** with C++17 support.
- **CMake** for the build system.
- **Boost** development library.
- **libssl** and **libcrypto**, both open-source libraries housed in the
  [OpenSSL Project repository](https://github.com/openssl/openssl).

The primary dependency, **libtorrent**, should be automatically downloaded and
built by the build script.

Run the following command to initiate the build process:

`cargo build --release`

or

`cargo build` if you want to build for development purposes.

## Building with Docker

A Docker file is also housed in this repository to build for Linux.

Building this Docker image produces a standalone executable of Torrenter:

`docker build --target prod .`

Replace the target **prod** with **dev** if you want a development build.

You can then copy the executable into a Linux environment with Windowing system
(such as X11 or Wayland) supported by
[https://github.com/rust-windowing/winit](winit).

Run the following command to create a Docker container out of your image and
 copy the executable from it:

```bash
id=$(docker create <image-name>)
docker cp $id:/torrenter .
```
