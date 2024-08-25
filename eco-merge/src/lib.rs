#![deny(clippy::all, clippy::pedantic)]

use camino::Utf8PathBuf;
use eco_cbz::{CbzReader, CbzWriter};
use glob::glob;
use tracing::warn;
use zip::{write::FileOptions, CompressionMethod};

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

    /// If not provided the images are stored as is (fastest), value must be between 0-9
    pub compression_level: Option<i64>,
}

#[allow(clippy::missing_errors_doc, clippy::needless_pass_by_value)]
pub fn merge(opts: MergeOptions) -> Result<()> {
    let mut merged_cbz_writer = CbzWriter::default();

    let mut file_options = FileOptions::<()>::default();
    if let Some(compression_level) = opts.compression_level {
        file_options = file_options.compression_level(Some(compression_level));
    } else {
        file_options = file_options.compression_method(CompressionMethod::Stored);
    }

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
            merged_cbz_writer.insert_with_file_options(image, file_options)?;

            Ok::<(), Error>(())
        })?;
    }

    merged_cbz_writer.write_to_path(opts.outdir.join(format!("{}.cbz", opts.name)))?;

    Ok(())
}
