use std::fmt;

use crate::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryEntityError {
    NotSpawned(EntityId),
    QueryDoesNotMatch(EntityId),
    AliasedMutability(EntityId),
}

impl fmt::Display for QueryEntityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSpawned(entity) => write!(formatter, "entity {entity} is not spawned"),
            Self::QueryDoesNotMatch(entity) => {
                write!(formatter, "query does not match entity {entity}")
            }
            Self::AliasedMutability(entity) => {
                write!(
                    formatter,
                    "entity {entity} was requested mutably more than once"
                )
            }
        }
    }
}

impl std::error::Error for QueryEntityError {}
