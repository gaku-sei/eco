#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// The path to the e-book file to view
    pub path: Utf8PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let path = Utf8PathBuf::try_from(dunce::canonicalize(args.path)?)?;

    eco_viewer::run(path)?;

    Ok(())
}
