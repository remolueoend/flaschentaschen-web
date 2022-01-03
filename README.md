# flaschentaschen-web
A small CLI tool for rendering websites on a remote [flaschentaschen](https://github.com/hzeller/flaschen-taschen) server.

It uses a headless chrome instance to get a stream of screen captures, transforms them to PPM file blobs and sends them to a remote flaschentaschen server.

## Setup
Following dependencies must be installed:
* Rust stable
* ImageMagick (version 7.0.x to 7.1.x)
* Clang (version 3.5 or higher) 

See https://github.com/nlfiedler/magick-rust for more info, which is responsible for these dependencies.

After that, clone this repository and navigate into the project root:
```sh
cargo build [--release]
```

You will then find the built cli executable at `<project-root>/target/release/flaschentaschen-web`

## Usage

```sh
$ ./flaschentaschen-web --help

USAGE:
    flaschentaschen-web [OPTIONS] --url <URL> --ft-endpoint <FT_ENDPOINT> --screen-width <SCREEN_WIDTH> --screen-height <SCREEN_HEIGHT>

OPTIONS:
        --ft-endpoint <FT_ENDPOINT>
            The host/port address of the target flaschentaschen server, e.g. localhost:1337

    -h, --help
            Print help information

        --screen-height <SCREEN_HEIGHT>
            The height of the LED screen (in pixels)

        --screen-width <SCREEN_WIDTH>
            The width of the LED screen (in pixels)

    -u, --url <URL>
            The URL of the website to screencast

    -v, --verbosity
            Set the level of verbosity (add multiple to increase level, e.g. -vvv)

    -V, --version
            Print version information
```

Make sure your flaschentaschen server and the website to render are accessible before running `flaschentaschen-web`.
