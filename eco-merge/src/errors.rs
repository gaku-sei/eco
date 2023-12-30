#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cbz error {0}")]
    Cbz(#[from] eco_cbz::Error),

    #[error("glob error {0}")]
    Glob(#[from] glob::GlobError),

    #[error("glob pattern error {0}")]
    GlobPattern(#[from] glob::PatternError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
