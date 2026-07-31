use thiserror::Error;

pub type ResourceResult<T> = std::result::Result<T, ResourceRegistryError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ResourceRegistryError {
    #[error("missing resource record for locator {locator}")]
    MissingRecordForLocator { locator: String },
    #[error("missing resource record for id {id}")]
    MissingRecordForId { id: String },
}
