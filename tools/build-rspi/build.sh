#!/usr/bin/env sh

# Builds the flaschentaschen-web binary for the RaspberryPi >=3 platform.
# Before running this script, run `./setup.sh` in this same directory
# to prepare the required docker image.
# After a successful build, the build assets can be found under
# $PROJECT_ROOT/targets/armv7-unknown-linux-gnueabihf/

SCRIPT=$(readlink -f "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
PROJECTPATH=$(dirname $(dirname "$SCRIPTPATH"))

docker run \
	--rm \
	-v $PROJECTPATH:/project \
	github.com/remolueoend/flaschentaschen-web/build/rspi:latest
