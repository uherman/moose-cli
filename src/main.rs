mod api;
mod menu;
mod player;
mod scrape;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// CLI för att titta på SVT:s "Den stora älgvandringen".
#[derive(Debug, Parser)]
#[command(name = "moose-cli", version, about)]
struct Cli {
    /// Kör mpv kopplad till terminalen (default är detach — mpv överlever att terminalen stängs).
    #[arg(short = 'a', long, global = true)]
    attach: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Starta live-sändningen (huvudkamera).
    Live,
    /// Starta live-sändningen från extrakamerorna.
    Extra,
    /// Spela ett specifikt videoId, eventuellt med start-offset i sekunder.
    Play {
        video_id: String,
        #[arg(long, default_value_t = 0)]
        start: u64,
    },
    /// Skriv ut HLS-URL för ett videoId till stdout (startar inte mpv).
    Url { video_id: String },
    /// Dumpa rå JSON från SVT:s video-API för inspektion.
    Debug { video_id: String },
    // TODO: Record { video_id, start, duration, output } — ffmpeg-inspelning, iteration 2.
    // TODO: Earlier — navigering till tidigare dygn, iteration 2.
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let detach = !cli.attach;
    match cli.command {
        None => menu::run(detach),
        Some(Command::Live) => play_page(scrape::LIVE_URL, detach),
        Some(Command::Extra) => play_page(scrape::EXTRA_URL, detach),
        Some(Command::Play { video_id, start }) => play_id(&video_id, start, detach),
        Some(Command::Url { video_id }) => print_url(&video_id),
        Some(Command::Debug { video_id }) => dump_debug(&video_id),
    }
}

fn play_page(page_url: &str, detach: bool) -> Result<()> {
    let video_id = scrape::video_id_from_page(page_url)?;
    play_id(&video_id, 0, detach)
}

fn play_id(video_id: &str, start: u64, detach: bool) -> Result<()> {
    let info = api::fetch(video_id)?;
    let hls = info.hls_url()?;
    player::play(hls, start, detach)
}

fn print_url(video_id: &str) -> Result<()> {
    let info = api::fetch(video_id)?;
    println!("{}", info.hls_url()?);
    Ok(())
}

fn dump_debug(video_id: &str) -> Result<()> {
    let raw = api::fetch_raw(video_id)?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}
