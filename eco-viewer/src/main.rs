#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use eco_cbz::CbzReader;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// The path to the cbz archive file to read
    pub archive_path: Utf8PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let mut cbz = CbzReader::from_path(args.archive_path)?;
    eco_viewer::run(&mut cbz)?;

    Ok(())
}
