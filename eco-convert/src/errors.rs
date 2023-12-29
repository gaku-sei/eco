#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error {0}")]
    Io(#[from] std::io::Error),

    #[error("cbz error {0}")]
    Cbz(#[from] eco_cbz::Error),

    #[error("mobi error {0}")]
    Mobi(#[from] mobi::MobiError),

    #[error("pdf error {0}")]
    Pdf(#[from] pdf::PdfError),

    #[error("ts parse error {0}")]
    TlParse(#[from] tl::ParseError),

    #[error("pack error {0}")]
    Pack(#[from] eco_pack::Error),

    #[error("invalid mobi version {0}")]
    InvalidMobiVersion(u32),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
