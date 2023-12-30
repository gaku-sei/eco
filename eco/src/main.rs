#![deny(clippy::all, clippy::pedantic)]

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use types::FileType;

use crate::errors::Result;
use crate::types::{Format, ReadingOrder};

mod errors;
mod types;

#[derive(Debug, Parser)]
#[clap(name = "eco", author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Args)]
struct GlobalOpts {
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    Convert {
        /// Path to the source file
        path: Utf8PathBuf,

        /// Source format
        #[clap(long, short)]
        from: Format,

        /// Dir to output images
        #[clap(long, short)]
        outdir: Utf8PathBuf,

        /// The archive name
        #[clap(long, short)]
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
    },
    Merge {
        /// A glob that matches all the archive to merge
        #[clap(short, long)]
        archives_glob: String,

        /// The output directory for the merged archive
        #[clap(short, long)]
        outdir: Utf8PathBuf,

        /// The merged archive name
        #[clap(short, long)]
        name: String,
    },
    Pack {
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
    },
    View {
        /// The path to the e-book file to view
        path: Utf8PathBuf,

        /// Type of the file
        #[clap(long = "type")]
        type_: Option<FileType>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    match args.command {
        Command::Convert {
            path,
            from,
            outdir,
            name,
            contrast,
            brightness,
            blur,
            autosplit,
            reading_order,
        } => eco_convert::convert(eco_convert::ConvertOptions {
            path,
            from: from.into(),
            outdir,
            name,
            contrast,
            brightness,
            blur,
            autosplit,
            reading_order: reading_order.into(),
        })?,
        Command::Merge {
            archives_glob,
            outdir,
            name,
        } => eco_merge::merge(eco_merge::MergeOptions {
            archives_glob,
            outdir,
            name,
        })?,
        Command::Pack {
            files_descriptor,
            outdir,
            name,
            contrast,
            brightness,
            blur,
            autosplit,
            reading_order,
        } => eco_pack::pack(eco_pack::PackOptions {
            files_descriptor,
            outdir,
            name,
            contrast,
            brightness,
            blur,
            autosplit,
            reading_order: reading_order.into(),
        })?,
        Command::View { path, type_ } => eco_view::view(eco_view::ViewOptions {
            path,
            type_: type_.map(Into::into),
        })?,
    }

    Ok(())
}
