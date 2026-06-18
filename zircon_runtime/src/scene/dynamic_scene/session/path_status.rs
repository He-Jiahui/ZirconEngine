use super::{RuntimeSessionArchiveError, RuntimeSessionArchiveManifest};

#[derive(Debug)]
pub enum RuntimeSessionArchivePathStatus {
    Missing,
    Available {
        manifest: RuntimeSessionArchiveManifest,
    },
    Invalid {
        error: RuntimeSessionArchiveError,
    },
}

impl RuntimeSessionArchivePathStatus {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn manifest(&self) -> Option<&RuntimeSessionArchiveManifest> {
        match self {
            Self::Available { manifest } => Some(manifest),
            Self::Missing | Self::Invalid { .. } => None,
        }
    }
}
