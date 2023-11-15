#![deny(clippy::all, clippy::pedantic)]

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use eco_cbz::{CbzReader, CbzWriter};
use glob::glob;
use tracing::warn;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// A glob that matches all the archive to merge
    #[clap(short, long)]
    pub archives_glob: String,
    /// The output directory for the merged archive
    #[clap(short, long)]
    pub outdir: Utf8PathBuf,
    /// The merged archive name
    #[clap(short, long)]
    pub name: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut merged_cbz_writer = CbzWriter::default();

    for path in glob(&args.archives_glob)? {
        let mut current_cbz = CbzReader::try_from_path(path?)?;

        current_cbz.try_for_each(|image| {
            let image = match image {
                Ok(image) => image,
                Err(err) => {
                    warn!("not a valid image: {err}");
                    return Ok::<(), anyhow::Error>(());
                }
            };
            merged_cbz_writer.insert(image)?;

            Ok::<(), anyhow::Error>(())
        })?;
    }

    merged_cbz_writer.write_to_path(args.outdir.join(format!("{}.cbz", args.name)))?;

    Ok(())
}
