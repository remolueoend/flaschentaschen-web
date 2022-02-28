#!/usr/bin/env sh

#
# Entrypoint script running in a docker container, cross-compiling the
# flaschentaschen-web binary for the RaspberryPi >=3 platform.
#

# assuming we run in a docker container where the project root is mounted at /project
cd /project
$HOME/.cargo/bin/cargo build --release --target armv7-unknown-linux-gnueabihf
