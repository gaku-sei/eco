#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::io::Cursor;

use camino::Utf8Path;
use eco_cbz::{
    image::{Image, ReadingOrder},
    CbzWriter,
};
use glob::glob;
use tracing::{debug, error};

use crate::errors::Result;

pub mod errors;

/// ## Errors
///
/// Fails when the glob is invalid, the paths are not utf-8, or the image can't be read and decoded
pub fn get_images_from_glob(glob_expr: impl AsRef<str>) -> Result<Vec<Image>> {
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
pub fn pack_imgs_to_cbz<'a>(
    imgs: Vec<Image>,
    contrast: Option<f32>,
    brightness: Option<i32>,
    blur: Option<f32>,
    autosplit: bool,
    reading_order: ReadingOrder,
) -> Result<CbzWriter<'a, Cursor<Vec<u8>>>> {
    let mut cbz_writer = CbzWriter::default();
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

        if img.is_landscape() && autosplit {
            debug!("splitting landscape file");
            let (img_left, img_right) = img.autosplit(reading_order);
            cbz_writer.insert_image(&img_left)?;
            cbz_writer.insert_image(&img_right)?;
        } else {
            cbz_writer.insert_image(&img)?;
        }
    }

    Ok(cbz_writer)
}
