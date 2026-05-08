use std::fmt;

use crate::scene::ecs::ComponentId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryAccessError {
    ConflictingComponentAccess { component_id: ComponentId },
}

impl fmt::Display for QueryAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingComponentAccess { component_id } => write!(
                f,
                "query accesses component {:?} mutably while it is already read or written",
                component_id
            ),
        }
    }
}

impl std::error::Error for QueryAccessError {}
