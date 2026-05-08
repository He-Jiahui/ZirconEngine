use std::fmt;

use crate::scene::ecs::{QueryAccessError, ResourceId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemParamError {
    Query(QueryAccessError),
    ConflictingResourceAccess { resource_id: ResourceId },
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
        }
    }
}

impl std::error::Error for SystemParamError {}

impl From<QueryAccessError> for SystemParamError {
    fn from(error: QueryAccessError) -> Self {
        Self::Query(error)
    }
}
