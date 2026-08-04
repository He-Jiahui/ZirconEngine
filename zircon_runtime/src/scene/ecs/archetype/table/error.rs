use thiserror::Error;

use crate::scene::ecs::ComponentId;
use crate::scene::EntityId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ArchetypeTableError {
    #[error("archetype table row is missing component column {component_id:?}")]
    MissingComponentColumn { component_id: ComponentId },
    #[error("archetype table row contains undeclared component column {component_id:?}")]
    UnexpectedComponentColumn { component_id: ComponentId },
    #[error("archetype table column {component_id:?} requires {expected_type}")]
    ComponentTypeMismatch {
        component_id: ComponentId,
        expected_type: &'static str,
    },
    #[error("archetype table row contains component column {component_id:?} more than once")]
    DuplicateComponentColumn { component_id: ComponentId },
    #[error("archetype table row {row} is outside its {len}-row table")]
    RowOutOfBounds { row: usize, len: usize },
    #[error("archetype table row {row} owns entity {actual}, not {expected}")]
    EntityRowMismatch {
        row: usize,
        expected: EntityId,
        actual: EntityId,
    },
}
