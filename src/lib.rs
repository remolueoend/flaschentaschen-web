use base64;
use eyre::{eyre, Result};
use headless_chrome::protocol::cdp::Page::{self, StartScreencastFormatOption};
use headless_chrome::{protocol::cdp::types::Event, Browser};
use image::pnm::{PNMSubtype, SampleEncoding};
use image::ImageOutputFormat;
use image::{load_from_memory_with_format, ImageFormat};
use log::{error, info, trace};
use serde_json;
use std::net::UdpSocket;
use std::sync::Mutex;
use std::{fmt::Display, sync::Arc};

pub mod cli;

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
    address: String,
    pub socket: UdpSocket,
}
impl FlaschenTaschen {
    /// Returns a new flaschentaschen instance for the given host/port.
    pub fn new(host_port: String) -> Result<FlaschenTaschen> {
        let socket = UdpSocket::bind("[::]:0")?; // bind local UDP socket
        socket.connect(&host_port)?;
        Ok(FlaschenTaschen {
            address: host_port,
            socket,
        })
    }

    /// Sends a given PPM byte slice this flaschentaschen server.
    pub fn send_ppm(&self, ppm: &[u8]) -> Result<usize> {
        self.socket
            .send(ppm)
            .map_err(|err| eyre!("failed to send PPM to {}: {}", self, err))
    }
}
impl Display for FlaschenTaschen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlaschenTaschen@{}", self.address)
    }
}

/// Starts the screencasting process by:
/// 1. spawing a new chrome instance
/// 2. navigating to the given URL
/// 3. attaching an event handler for incoming frames which forwards them to the given `on_frame` callback.
/// This method will return the created browser instance. It is important to keep the returned instance in scope.
/// If it goes out of scope or the main thread terminates, the browser will be stopped too and screencasting halts.
///
/// This function will call the provided callback for each received frame together with the given static context.
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
    let event_listener = move |event: &Event| match event {
        Event::PageScreencastFrame(frame) => {
            let mut current_err_count = consecutive_err_count.lock().unwrap();
            trace!(
                "got frame: {}",
                frame.params.metadata.timestamp.expect("missing timestamp")
            );
            // we do catch potential errors but only log them and continue with the next frame.
            // if we get more than a fixed threshold of consecutive errors, we stop the screencasting
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
    };
    map_err(
        tab.add_event_listener(Arc::new(event_listener)),
        "Failed to attach event listener to tab",
    )?;

    // tell chrome to start screencasting:
    map_err(
        tab.call_method(Page::StartScreencast {
            every_nth_frame: Some(1),
            format: Some(StartScreencastFormatOption::Jpeg),
            max_height: Some(opts.height),
            max_width: Some(opts.width),
            quality: Some(100),
        }),
        "failed to start screencasting",
    )?;

    Ok(browser)
}

/// Accepts a base64 encoded string of a JPEG image and returns its PPM counterpart as a byte vector.
pub fn get_ppm_from_jpeg(base64_str: &String) -> Result<Vec<u8>> {
    let buffer = base64::decode(base64_str)?;
    let input_image = load_from_memory_with_format(buffer.as_slice(), ImageFormat::Jpeg)?;

    let mut output: Vec<u8> = Vec::new();
    input_image.write_to(
        &mut output,
        // PPM with magic P6:
        ImageOutputFormat::Pnm(PNMSubtype::Pixmap(SampleEncoding::Binary)),
    )?;

    Ok(output)
}
