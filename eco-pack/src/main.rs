#![deny(clippy::all, clippy::pedantic)]

use std::{env, fs::create_dir};

use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use eco_cbz::image::ReadingOrder;
use eco_pack::{get_images_from_glob, pack_imgs_to_cbz};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// A glob that matches all the files to pack
    files_descriptor: String,
    /// The output directory for the merged archive
    #[clap(short, long, default_value = "./")]
    outdir: Utf8PathBuf,
    /// The merged archive name
    #[clap(short, long)]
    name: String,
    /// Adjust images contrast
    #[clap(long)]
    contrast: Option<f32>,
    /// Adjust images brightness
    #[clap(long)]
    brightness: Option<i32>,
    /// Blur image (slow with big numbers)
    #[clap(long)]
    blur: Option<f32>,
    /// Automatically split landscape images into 2 pages
    #[clap(long, action)]
    autosplit: bool,
    /// Reading order
    #[clap(long, default_value_t = ReadingOrder::Rtl)]
    reading_order: ReadingOrder,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let Ok(current_dir) = Utf8PathBuf::from_path_buf(env::current_dir()?) else {
        bail!("current dir is not a valid utf-8 path");
    };
    let outdir = current_dir.join(&args.outdir);
    if !outdir.exists() {
        create_dir(&*outdir)?;
    }
    let imgs = get_images_from_glob(args.files_descriptor)?;

    let cbz_writer = pack_imgs_to_cbz(
        imgs,
        args.contrast,
        args.brightness,
        args.blur,
        args.autosplit,
        args.reading_order,
    )?;

    cbz_writer.write_to_path(outdir.join(format!("{}.cbz", args.name)))?;

    Ok(())
}
