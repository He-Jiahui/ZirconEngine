use thiserror::Error;

use crate::asset::ReferenceResolutionError;

#[derive(Debug, Error)]
pub enum ProjectDocumentError {
    #[error("project document TOML deserialization failed: {source}")]
    Deserialize {
        #[source]
        source: toml::de::Error,
    },
    #[error("project document TOML serialization failed: {source}")]
    Serialize {
        #[source]
        source: toml::ser::Error,
    },
    #[error(transparent)]
    Reference(#[from] ReferenceResolutionError),
    #[error("unsupported project document schema: {message}")]
    Schema { message: String },
}

impl From<toml::de::Error> for ProjectDocumentError {
    fn from(source: toml::de::Error) -> Self {
        Self::Deserialize { source }
    }
}

impl From<toml::ser::Error> for ProjectDocumentError {
    fn from(source: toml::ser::Error) -> Self {
        Self::Serialize { source }
    }
}
