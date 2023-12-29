#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("convert error {0}")]
    Convert(#[from] eco_convert::Error),

    #[error("merge error {0}")]
    Merge(#[from] eco_merge::Error),

    #[error("pack error {0}")]
    Pack(#[from] eco_pack::Error),

    #[error("view error {0}")]
    View(#[from] eco_view::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
