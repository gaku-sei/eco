#![deny(clippy::all, clippy::pedantic)]

use std::fs;

use ::mobi::Mobi;
use ::pdf::file::FileOptions as PdfFileOptions;
use camino::Utf8PathBuf;
use eco_cbz::image::ReadingOrder;
use eco_pack::pack_imgs_to_cbz;
use tracing::info;

pub use crate::errors::{Error, Result};
pub use crate::mobi::convert_to_imgs as mobi_to_imgs;
pub use crate::pdf::convert_to_imgs as pdf_to_imgs;

pub mod errors;
mod mobi;
mod pdf;
mod utils;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Mobi,
    Azw3,
    Pdf,
}

#[derive(Debug)]
pub struct ConvertOptions {
    /// Path to the source file
    pub path: Utf8PathBuf,

    /// Source format
    pub from: Format,

    /// Dir to output images
    pub outdir: Utf8PathBuf,

    /// The archive name
    pub name: String,

    /// Adjust images contrast
    pub contrast: Option<f32>,

    /// Adjust images brightness
    pub brightness: Option<i32>,

    /// Blur image (slow with big numbers)
    pub blur: Option<f32>,

    /// Automatically split landscape images into 2 pages
    pub autosplit: bool,

    /// Reading order
    pub reading_order: ReadingOrder,

    /// If not provided the images are stored as is (fastest), value must be between 0-9
    pub compression_level: Option<i64>,
}

#[allow(clippy::missing_errors_doc)]
pub fn convert(opts: ConvertOptions) -> Result<()> {
    fs::create_dir_all(&opts.outdir)?;
    let cbz_writer = match opts.from {
        Format::Mobi | Format::Azw3 => {
            let mobi = Mobi::from_path(opts.path)?;
            let imgs = mobi_to_imgs(&mobi)?;
            info!("found {} imgs", imgs.len());
            pack_imgs_to_cbz(
                imgs,
                opts.contrast,
                opts.brightness,
                opts.blur,
                opts.autosplit,
                opts.reading_order,
                opts.compression_level,
            )?
        }
        Format::Pdf => {
            let pdf = PdfFileOptions::cached().open(opts.path)?;
            let imgs = pdf_to_imgs(&pdf)?;
            info!("found {} imgs", imgs.len());
            pack_imgs_to_cbz(
                imgs,
                opts.contrast,
                opts.brightness,
                opts.blur,
                opts.autosplit,
                opts.reading_order,
                opts.compression_level,
            )?
        }
    };

    cbz_writer.write_to_path(opts.outdir.join(format!("{}.cbz", opts.name)))?;

    Ok(())
}
