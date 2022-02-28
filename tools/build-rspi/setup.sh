#!/usr/bin/env sh

# creates the docker image required to build the flaschentaschen-web binary
# for the RaspberryPi >= 3 platform.
# This script generates a new image with the tag: github.com/remolueoend/flaschentaschen-web/build-rspi:latest
# Afterwards, use the script `./build.sh` in this same folder to start the build.

SCRIPT=$(readlink -f "$0")
SCRIPTPATH=$(dirname "$SCRIPT")

(cd $SCRIPTPATH && docker build -t github.com/remolueoend/flaschentaschen-web/build/rspi .)
