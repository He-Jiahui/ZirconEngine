#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReflectFieldIdParseError {
    #[error("reflect field ID is not a UUID: {source}")]
    InvalidUuid { source: uuid::Error },
    #[error("reflect field ID must not be nil")]
    Nil,
}
