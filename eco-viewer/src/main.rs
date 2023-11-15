#![deny(clippy::all, clippy::pedantic)]

use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use eco_viewer::FileType;

#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
    /// The path to the e-book file to view
    pub path: Utf8PathBuf,

    /// Type of the file
    #[clap(long = "type")]
    pub type_: Option<FileType>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let path = Utf8PathBuf::try_from(dunce::canonicalize(args.path)?)?;
    let Some(file_type) = args
        .type_
        .or_else(|| path.extension().and_then(|ext| ext.parse().ok()))
    else {
        bail!("unknown file type");
    };

    eco_viewer::run(path, file_type)?;

    Ok(())
}
