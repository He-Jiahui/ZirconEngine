use std::fmt;

use crate::scene::ecs::ComponentId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryAccessError {
    ConflictingComponentAccess { component_id: ComponentId },
    ProtectedDerivedComponentWrite { component: &'static str },
    ProtectedAuthoredComponentWrite { component: &'static str },
}

impl fmt::Display for QueryAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingComponentAccess { component_id } => write!(
                f,
                "query accesses component {:?} mutably while it is already read or written",
                component_id
            ),
            Self::ProtectedDerivedComponentWrite { component } => {
                write!(
                    f,
                    "query cannot mutably access World-owned derived component {component}"
                )
            }
            Self::ProtectedAuthoredComponentWrite { component } => {
                write!(
                    f,
                    "query cannot mutably access Scene-owned authored component {component}"
                )
            }
        }
    }
}

impl std::error::Error for QueryAccessError {}
