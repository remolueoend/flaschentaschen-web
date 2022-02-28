use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct CliArgs {
    /// The URL of the website to screencast
    #[clap(short = 'u', long)]
    pub url: String,

    /// The address of the target flaschentaschen server, e.g. localhost:1337
    #[clap(short = 'f', long)]
    pub ft_endpoint: String,

    /// The width of the LED screen (in pixels)
    #[clap(short = 'w', long)]
    pub screen_width: u32,

    /// The height of the LED screen (in pixels)
    #[clap(short = 'h', long)]
    pub screen_height: u32,

    /// Set the level of verbosity (add multiple to increase level, e.g. -vvv)
    #[clap(short = 'v', long, parse(from_occurrences))]
    pub verbosity: u64,
}
