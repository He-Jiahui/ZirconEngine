use thiserror::Error;

pub type ResourceResult<T> = std::result::Result<T, ResourceRegistryError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ResourceRegistryError {
    #[error("missing resource record for locator {locator}")]
    MissingRecordForLocator { locator: String },
    #[error("missing resource record for id {id}")]
    MissingRecordForId { id: String },
    #[error("resource locator {locator} is already owned by {existing_id}, not {requested_id}")]
    LocatorOccupied {
        locator: String,
        existing_id: String,
        requested_id: String,
    },
    #[error(
        "resource {id} locator change from {current_locator} to {requested_locator} requires rename"
    )]
    ExplicitRenameRequired {
        id: String,
        current_locator: String,
        requested_locator: String,
    },
    #[error("resource {id} kind is {current_kind:?}, not {requested_kind:?}")]
    KindConflict {
        id: String,
        current_kind: crate::core::resource::ResourceKind,
        requested_kind: crate::core::resource::ResourceKind,
    },
    #[error("resource {id} cannot transition from {current_state:?} to {requested_state:?}")]
    InvalidStateTransition {
        id: String,
        current_state: crate::core::resource::ResourceState,
        requested_state: crate::core::resource::ResourceState,
    },
    #[error(
        "resource {id} revision conflict: expected {expected_revision}, actual {actual_revision:?}"
    )]
    RevisionConflict {
        id: String,
        expected_revision: u64,
        actual_revision: Option<u64>,
    },
}
