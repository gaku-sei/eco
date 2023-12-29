#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {0}")]
    Generic(String),

    #[error("glob error: {0}")]
    Glob(#[from] glob::GlobError),

    #[error("glob pattern error: {0}")]
    GlobPattern(#[from] glob::PatternError),

    #[error("cbz error: {0}")]
    Cbz(#[from] eco_cbz::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
