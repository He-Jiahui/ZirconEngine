use std::fmt;

use crate::scene::ecs::{QueryAccessError, ResourceId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemParamError {
    Query(QueryAccessError),
    ConflictingResourceAccess { resource_id: ResourceId },
    ConflictingEventAccess { type_name: &'static str },
    ConflictingMessageAccess { type_name: &'static str },
    MissingResource { type_name: &'static str },
}

impl fmt::Display for SystemParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(error) => write!(f, "{error}"),
            Self::ConflictingResourceAccess { resource_id } => write!(
                f,
                "system accesses resource {:?} mutably while it is already read or written",
                resource_id
            ),
            Self::ConflictingEventAccess { type_name } => write!(
                f,
                "system accesses event {type_name} mutably while events are already read or written"
            ),
            Self::ConflictingMessageAccess { type_name } => write!(
                f,
                "system accesses message {type_name} mutably while messages are already read or written"
            ),
            Self::MissingResource { type_name } => {
                write!(f, "requested missing scene resource {type_name}")
            }
        }
    }
}

impl std::error::Error for SystemParamError {}

impl From<QueryAccessError> for SystemParamError {
    fn from(error: QueryAccessError) -> Self {
        Self::Query(error)
    }
}
