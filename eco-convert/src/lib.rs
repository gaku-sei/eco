#![deny(clippy::all, clippy::pedantic)]

use std::fs;

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
}

#[allow(clippy::missing_errors_doc)]
pub fn convert(opts: ConvertOptions) -> Result<()> {
    fs::create_dir_all(&opts.outdir)?;
    let imgs = match opts.from {
        Format::Mobi | Format::Azw3 => mobi_to_imgs(opts.path)?,
        Format::Pdf => pdf_to_imgs(opts.path)?,
    };
    info!("found {} imgs", imgs.len());

    let cbz_writer = pack_imgs_to_cbz(
        imgs,
        opts.contrast,
        opts.brightness,
        opts.blur,
        opts.autosplit,
        opts.reading_order,
    )?;

    cbz_writer.write_to_path(opts.outdir.join(format!("{}.cbz", opts.name)))?;

    Ok(())
}
