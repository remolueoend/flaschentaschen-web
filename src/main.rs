use clap::Parser;
use color_eyre::eyre::Result;
use flaschentaschen_web::{cli::CliArgs, ScreencastOptions};
use flaschentaschen_web::{get_ppm_from_jpeg, start_screencasting, FlaschenTaschen};
use headless_chrome::protocol::cdp::Page;
use log::info;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;

/// handles an incoming screencast frame from the browser by converting it to PPM
/// and sending it to the flaschentaschen server.
fn on_screencast_frame(
    frame: &Page::events::ScreencastFrameEvent,
    flaschentaschen: &FlaschenTaschen,
) -> Result<()> {
    let ppm = get_ppm_from_jpeg(&frame.params.data)?;
    flaschentaschen.send_ppm(ppm.as_slice())?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = CliArgs::parse();
    loggerv::init_with_verbosity(args.verbosity)?;

    let screencast_opts = ScreencastOptions {
        url: args.url,
        width: args.screen_width,
        height: args.screen_height,
    };

    // leak is fine here: this context instance is created once and passed as reference to `on_screencast_frame`.
    // As soon as main exits, this memory reference is not needed anymore because the thread handling the browser tab event is haltet too.
    let flaschentaschen: &'static mut FlaschenTaschen =
        Box::leak(Box::new(FlaschenTaschen::new(args.ft_endpoint)?));

    let browser = start_screencasting(screencast_opts, on_screencast_frame, flaschentaschen)?;
    info!(
        "started chrome instance with process id {}",
        browser.get_process_id().unwrap()
    );

    // wait for a SIGINT signal
    let mut signals = Signals::new(&[SIGINT])?;
    let signal_thread = thread::spawn(move || {
        for sig in signals.forever() {
            info!("Received signal {}, exiting...", sig);
            return;
        }
    });

    // Important: We need to make sure to keep this process busy.
    // If `browser` leaves its scope, the browser instance will be stopped and screencasting halts.
    // We do this by waiting for a SIGINT signal in a separate thread and join it:
    signal_thread
        .join()
        .expect("failed to wait for signal thread");

    Ok(())
}
