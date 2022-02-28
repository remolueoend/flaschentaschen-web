# generates a docker image for building the flaschentaschen-web binarary
# for the RaspberryPI >= 3 platform

FROM debian:latest

WORKDIR /

RUN apt-get update
RUN apt-get -y install curl gcc-arm-linux-gnueabihf libssl-dev build-essential pkg-config
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN $HOME/.cargo/bin/rustup target add armv7-unknown-linux-gnueabihf

ADD ./entrypoint.sh .

ENTRYPOINT [ "/entrypoint.sh" ]
