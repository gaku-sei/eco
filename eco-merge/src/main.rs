#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use eco_cbz::{CbzReader, CbzWriter};
use glob::glob;

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
    let args = Args::parse();

    let mut merged_cbz_writer = CbzWriter::default();

    for path in glob(&args.archives_glob)? {
        let mut current_cbz = CbzReader::try_from_path(path?)?;

        current_cbz.try_for_each(|image| {
            merged_cbz_writer.insert(image?)?;

            Ok::<(), anyhow::Error>(())
        })?;
    }

    merged_cbz_writer.write_to_path(args.outdir.join(format!("{}.cbz", args.name)))?;

    Ok(())
}
