use std::fmt;

use crate::scene::ecs::{QueryAccessError, ResourceId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemParamError {
    TupleElement {
        index: usize,
        parameter_type: &'static str,
        source: Box<SystemParamError>,
    },
    MultipleDeferredCommandParams,
    Query(QueryAccessError),
    ConflictingResourceAccess {
        resource_id: ResourceId,
    },
    ConflictingEventAccess {
        type_name: &'static str,
    },
    ConflictingMessageAccess {
        type_name: &'static str,
    },
    EventReaderLeaseExhausted {
        type_name: &'static str,
    },
    MissingResource {
        type_name: &'static str,
    },
}

impl fmt::Display for SystemParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TupleElement {
                index,
                parameter_type,
                source,
            } => write!(
                f,
                "system tuple parameter {index} ({parameter_type}) failed: {source}"
            ),
            Self::MultipleDeferredCommandParams => {
                write!(f, "a system may own at most one deferred command lane")
            }
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
            Self::EventReaderLeaseExhausted { type_name } => {
                write!(f, "event reader lease capacity exhausted for {type_name}")
            }
            Self::MissingResource { type_name } => {
                write!(f, "requested missing scene resource {type_name}")
            }
        }
    }
}

impl std::error::Error for SystemParamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TupleElement { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl SystemParamError {
    pub(crate) fn in_tuple(self, index: usize, parameter_type: &'static str) -> Self {
        Self::TupleElement {
            index,
            parameter_type,
            source: Box::new(self),
        }
    }
}

impl From<QueryAccessError> for SystemParamError {
    fn from(error: QueryAccessError) -> Self {
        Self::Query(error)
    }
}
