use thiserror::Error;

/// Failure to construct a canonical persistent asset reference.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AssetRefError {
    #[error("asset subpath cannot be empty")]
    EmptySubPath,
    #[error("asset subpath cannot contain the fragment delimiter '#'")]
    FragmentDelimiterInSubPath,
    #[error("asset subpath cannot contain a control character at byte index {index}")]
    ControlCharacterInSubPath { index: usize },
}
