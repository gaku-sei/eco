#![deny(clippy::all, clippy::pedantic)]

use std::{
    env,
    fs::create_dir_all,
    io::{BufRead, Cursor, Seek},
};

use camino::{Utf8Path, Utf8PathBuf};
use eco_cbz::{
    image::{Image, ImageFile, ReadingOrder},
    CbzWriter,
};
use glob::glob;
use tracing::{debug, error};
use zip::{write::FileOptions, CompressionMethod};

pub use crate::errors::{Error, Result};

pub mod errors;

/// ## Errors
///
/// Fails when the glob is invalid, the paths are not utf-8, or the image can't be read and decoded
pub fn get_images_from_glob(glob_expr: impl AsRef<str>) -> Result<Vec<ImageFile>> {
    let paths = glob(glob_expr.as_ref())?;
    let mut imgs = Vec::new();

    for path in paths {
        let path = path?;
        let Some(path) = Utf8Path::from_path(&path) else {
            error!("{path:?} is not a valid utf-8 path");
            continue;
        };
        imgs.push(Image::open(path)?);
    }

    Ok(imgs)
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn pack_imgs_to_cbz<R: BufRead + Seek>(
    imgs: Vec<Image<R>>,
    contrast: Option<f32>,
    brightness: Option<i32>,
    blur: Option<f32>,
    autosplit: bool,
    reading_order: ReadingOrder,
    compression_level: Option<i32>,
) -> Result<CbzWriter<Cursor<Vec<u8>>>> {
    let mut cbz_writer = CbzWriter::default();

    let mut file_options = FileOptions::default();
    if let Some(compression_level) = compression_level {
        file_options = file_options.compression_level(Some(compression_level));
    } else {
        file_options = file_options.compression_method(CompressionMethod::Stored);
    }

    for mut img in imgs {
        if let Some(contrast) = contrast {
            img = img.set_contrast(contrast);
        }
        if let Some(brightness) = brightness {
            img = img.set_brightness(brightness);
        }
        if let Some(blur) = blur {
            img = img.set_blur(blur);
        }
        if autosplit && img.is_landscape() {
            debug!("splitting landscape file");
            let (img_left, img_right) = img.autosplit(reading_order);
            cbz_writer.insert_with_file_options(img_left, file_options)?;
            cbz_writer.insert_with_file_options(img_right, file_options)?;
        } else {
            cbz_writer.insert_with_file_options(img, file_options)?;
        }
    }

    Ok(cbz_writer)
}

#[derive(Debug)]
pub struct PackOptions {
    /// A glob that matches all the files to pack
    pub files_descriptor: String,

    /// The output directory for the merged archive
    pub outdir: Utf8PathBuf,

    /// The merged archive name
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
    pub compression_level: Option<i32>,
}

#[allow(clippy::missing_errors_doc)]
pub fn pack(opts: PackOptions) -> Result<()> {
    let Ok(current_dir) = Utf8PathBuf::from_path_buf(env::current_dir()?) else {
        return Err(Error::Generic(
            "current dir is not a valid utf8 path".to_string(),
        ));
    };
    let outdir = current_dir.join(&opts.outdir);
    if !outdir.exists() {
        create_dir_all(&*outdir)?;
    }
    let imgs = get_images_from_glob(opts.files_descriptor)?;

    let cbz_writer = pack_imgs_to_cbz(
        imgs,
        opts.contrast,
        opts.brightness,
        opts.blur,
        opts.autosplit,
        opts.reading_order,
        opts.compression_level,
    )?;

    cbz_writer.write_to_path(outdir.join(format!("{}.cbz", opts.name)))?;

    Ok(())
}
