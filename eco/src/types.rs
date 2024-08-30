use std::fmt::Display;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ReadingOrder {
    Rtl,
    Ltr,
}

impl From<ReadingOrder> for eco_cbz::ReadingOrder {
    fn from(value: ReadingOrder) -> Self {
        match value {
            ReadingOrder::Ltr => Self::Ltr,
            ReadingOrder::Rtl => Self::Rtl,
        }
    }
}

impl Display for ReadingOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ltr => "ltr",
                Self::Rtl => "rtl",
            }
        )
    }
}

// TODO: Format and FileType can, and should, be merged together, but the underlying should support them
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Mobi,
    Azw3,
    Pdf,
}

impl From<Format> for eco_convert::Format {
    fn from(value: Format) -> Self {
        match value {
            Format::Azw3 => Self::Azw3,
            Format::Mobi => Self::Mobi,
            Format::Pdf => Self::Pdf,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FileType {
    #[clap(name = "cbz")]
    Cbz,
    #[clap(skip, name = "epub")]
    EPub,
}

impl From<FileType> for eco_view::FileType {
    fn from(value: FileType) -> Self {
        match value {
            FileType::Cbz => Self::Cbz,
            FileType::EPub => Self::Epub,
        }
    }
}
