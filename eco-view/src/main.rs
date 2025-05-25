#![deny(clippy::all, clippy::pedantic)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use camino::Utf8PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "eco-view", author, version, about, long_about = None)]
struct Args {
    /// The path to the e-book file to view
    path: Utf8PathBuf,
}

fn main() -> Result<(), eco_view::Error> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt::init();
    } // TODO: else use tracing appender to log into file
    let args = Args::parse();

    eco_view::view(eco_view::ViewOptions {
        path: args.path,
        type_: None,
    })?;

    Ok(())
}
