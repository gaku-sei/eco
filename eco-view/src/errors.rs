#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cbz error: {0}")]
    Cbz(#[from] eco_cbz::Error),

    #[error("epub doc error: {0}")]
    EpubDoc(#[from] epub::doc::DocError),

    #[error("html parse error: {0}")]
    HtmlParse(#[from] tl::ParseError),

    #[error("html bytes conversion error: {0}")]
    HtmlBytesConversion(#[from] tl::errors::SetBytesError),

    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("invalid file type: {0}")]
    InvalidFileType(String),

    #[error("page not found: {0}")]
    PageNotFound(usize),

    #[error("invalid non utf8 path provided")]
    InvalidNonUtf8Path,

    #[error("unknown file type provided")]
    UnknownFileType,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
