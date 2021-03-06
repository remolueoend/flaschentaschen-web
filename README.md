# flaschentaschen-web
A small CLI tool for rendering websites on a remote [flaschentaschen](https://github.com/hzeller/flaschen-taschen) server:

![image](https://user-images.githubusercontent.com/7881606/147987529-3a744e97-ee9c-4d20-a050-5111236084de.png)

It uses a headless chrome instance to get a stream of screen captures, transforms them to PPM file blobs and sends them to a remote flaschentaschen server.

## Setup
Following build dependencies must be installed:
* [Rust stable toolchain](https://www.rust-lang.org/tools/install)

After that, clone this repository, navigate into the project root and run:
```sh
cargo build [--release]
```

You will then find the CLI executable at `<project-root>/target/release/flaschentaschen-web`

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

Make sure your flaschentaschen server and the website to render are accessible before running `flaschentaschen-web`. Big thanks to [Henner Zeller](https://github.com/hzeller) for his amazing projects!


## Development
### Cross-compile for the RaspberryPi platform
Follow the README at [./tools/build-rspi](./tools/build-rspi)
