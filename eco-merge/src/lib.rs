#![deny(clippy::all, clippy::pedantic)]

use camino::Utf8PathBuf;
use eco_cbz::{CbzReader, CbzWriter};
use glob::glob;
use tracing::warn;

pub use crate::errors::{Error, Result};

pub mod errors;

#[derive(Debug)]
pub struct MergeOptions {
    /// A glob that matches all the archive to merge
    pub archives_glob: String,

    /// The output directory for the merged archive
    pub outdir: Utf8PathBuf,

    /// The merged archive name
    pub name: String,
}

#[allow(clippy::missing_errors_doc, clippy::needless_pass_by_value)]
pub fn merge(opts: MergeOptions) -> Result<()> {
    let mut merged_cbz_writer = CbzWriter::default();

    for path in glob(&opts.archives_glob)? {
        let mut current_cbz = CbzReader::try_from_path(path?)?;

        current_cbz.try_for_each(|image| {
            let image = match image {
                Ok(image) => image,
                Err(err) => {
                    warn!("not a valid image: {err}");
                    return Ok::<(), Error>(());
                }
            };
            merged_cbz_writer.insert(image)?;

            Ok::<(), Error>(())
        })?;
    }

    merged_cbz_writer.write_to_path(opts.outdir.join(format!("{}.cbz", opts.name)))?;

    Ok(())
}
