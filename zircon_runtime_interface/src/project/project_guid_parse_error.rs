use thiserror::Error;

/// Rejects malformed or non-unique persisted project GUID values.
#[derive(Debug, Error)]
pub enum ProjectGuidParseError {
    #[error("project GUID is not a UUID: {source}")]
    InvalidUuid {
        #[source]
        source: uuid::Error,
    },
    #[error("project GUID must not be the nil UUID")]
    Nil,
}
