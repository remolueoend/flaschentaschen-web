use std::sync::{Mutex, Once};
use std::{fmt::Display, sync::Arc};

use base64;

use eyre::{eyre, Result};

use headless_chrome::protocol::cdp::Page::{self, StartScreencastFormatOption};
use headless_chrome::{protocol::cdp::types::Event, Browser};
use log::{error, info, trace};
use magick_rust::{magick_wand_genesis, MagickWand};
use serde_json;
use std::net::UdpSocket;

pub mod cli;

/// Used to make sure we initialize Magick only once
static INIT_MAGICK: Once = Once::new();

/// Wraps a Result value with a compatible error type and returns a new result with an eyre-compatible Report error type.
/// The given message is prepended to the display result of the original error.
fn map_err<T, E: Display>(value: std::result::Result<T, E>, msg: &str) -> Result<T> {
    value.map_err(|err| eyre!("{}: {}", msg, err))
}

/// Screencast options passed to `start_screencasting`
pub struct ScreencastOptions {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

/// Provides a connection context to a flaschentaschen server
pub struct FlaschenTaschen {
    pub socket: UdpSocket,
}
impl FlaschenTaschen {
    pub fn new(host_port: String) -> Result<FlaschenTaschen> {
        let socket = UdpSocket::bind("[::]:0")?; // bind local UDP socket
        socket.connect(host_port)?;
        Ok(FlaschenTaschen { socket })
    }
}

/// Starts the screencasting process by:
/// 1. spawing a new chrome instance
/// 2. navigating to the given URL
/// 3. attaching an event handler for incoming frames which forwards them to the flaschentaschen server.
/// This method will return the created browser instance. It is important to keep the returned instance in scope.
/// If it goes out of scope or the main thread terminates, the browser will be stopped too and screencasting halts.
///
/// This function will call the provided callback for each received frame together with the given context.
/// This is necessary because the callback will run in a separate thread.
pub fn start_screencasting<F, C>(
    opts: ScreencastOptions,
    on_frame: F,
    on_frame_context: &'static C,
) -> Result<Browser>
where
    C: Send + Sync,
    F: 'static + Fn(&Page::events::ScreencastFrameEvent, &'static C) -> Result<()> + Send + Sync,
{
    info!(
        "starting chrome in headless mode with dimensions {}x{}",
        opts.width, opts.height
    );

    // open the browser on the provided URL:
    let browser = map_err(
        Browser::new(headless_chrome::LaunchOptions {
            headless: true,
            window_size: Some((opts.width, opts.height)),
            ..Default::default()
        }),
        "Failed to launch browser",
    )?;
    let tab = map_err(browser.wait_for_initial_tab(), "Could not open new tab")?;
    map_err(
        tab.navigate_to(opts.url.as_str()),
        format!("Could not navigate to {}", opts.url).as_str(),
    )?;
    let closure_tab = tab.clone();

    // register the event handler for incoming screencast frames.
    // `consecutive_err_count` will count consecutive errors while handling incoming frames to stop screencasting
    // as soon as a threshold is reached.
    let consecutive_err_count = Arc::new(Mutex::new(0));
    map_err(
        tab.add_event_listener(Arc::new(move |event: &Event| match event {
            Event::PageScreencastFrame(frame) => {
                let mut current_err_count = consecutive_err_count.lock().unwrap();
                trace!(
                    "got frame: {}",
                    frame.params.metadata.timestamp.expect("missing timestamp")
                );
                // we do catch potential errors but only log them and continue with the next frame.
                // if we get more than 10 consecutive errors, we stop the screencasting
                let callback_result = on_frame(frame, on_frame_context);
                if callback_result.is_ok() {
                    *current_err_count = 0;
                } else {
                    *current_err_count += 1;
                    error!(
                        "frame handler failed (consecutive errors: {}): {}",
                        current_err_count,
                        callback_result.unwrap_err()
                    );
                }

                // TODO: for some reason, UdpSocket.send will return Ok() even if the server is not reachable.
                // this will wrongly reset the consecutive error count.
                if *current_err_count > 1000 {
                    let _ = closure_tab
                        .call_method(Page::StopScreencast(Some(serde_json::value::Value::Null)));
                } else {
                    let _ = closure_tab.call_method(Page::ScreencastFrameAck {
                        session_id: frame.params.session_id,
                    });
                }
            }
            _ => {}
        })),
        "Failed to subscribe to event",
    )?;

    // tell chrome to start screencasting:
    map_err(
        tab.call_method(Page::StartScreencast {
            every_nth_frame: Some(1),
            format: Some(StartScreencastFormatOption::Png),
            max_height: Some(opts.height),
            max_width: Some(opts.width),
            quality: Some(100),
        }),
        "failed to start screencasting",
    )?;

    Ok(browser)
}

/// Accepts a base64 encoded string of a PNG image and returns its PPM counterpart as a byte vector.
pub fn get_ppm_from_png(base64_str: &String) -> Result<Vec<u8>> {
    INIT_MAGICK.call_once(|| {
        magick_wand_genesis();
    });
    let buffer = base64::decode(base64_str)?;
    let wand = MagickWand::new();
    wand.read_image_blob(buffer)
        .map_err(|err| eyre!("failed to read base64 PNG: {}", err))?;

    let output = wand
        .write_image_blob("ppm")
        .map_err(|err| eyre!("failed to write to ppm: {}", err))?;

    Ok(output)
}

/// Sends a given PPM byte slice to the given flaschentaschen server.
pub fn send_ppm_to_flaschentaschen(ppm: &[u8], flaschentaschen: &FlaschenTaschen) -> Result<usize> {
    flaschentaschen
        .socket
        .send(ppm)
        .map_err(|err| eyre!("failed to send PPM to flaschentaschen: {}", err))
}