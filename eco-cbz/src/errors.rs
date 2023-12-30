use zip::result::ZipError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error {0}")]
    IO(#[from] std::io::Error),

    #[error("zip error {0}")]
    Zip(#[from] ZipError),

    #[error("cbz file size couldn't be converted")]
    CbzFileSizeConversion,

    #[error("cbz file name is empty")]
    CbzFileNameEmpty,

    #[error("cbz file invalid index {0}")]
    CbzFileInvalidIndex(String),

    #[error("file at index {0} not found in cbz")]
    CbzNotFound(usize),

    #[error("cbz is too large, it can contain a maximum of {0} files")]
    CbzTooLarge(usize),

    #[error("cbz file insertion's extension not provided")]
    CbzInsertionNoExtension,

    #[error("cbz file insertion: no bytes set")]
    CbzInsertionNoBytes,

    #[error("cbz metadata is too large: {0} > 65,535")]
    CbzMetadataSize(usize),

    #[error("image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("unknown image format error")]
    UnknownImageFormat,

    #[cfg(feature = "metadata")]
    #[error("metadata error: {0}")]
    MetadataFormat(#[from] serde_json::Error),

    #[cfg(feature = "metadata")]
    #[error("metadata value error: {0}")]
    MetadataValue(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
