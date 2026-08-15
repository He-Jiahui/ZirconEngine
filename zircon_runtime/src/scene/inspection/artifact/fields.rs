use std::collections::BTreeMap;
use std::sync::Arc;

use crate::scene::{EntityId, World};

use super::super::snapshot::build_inspection_fields;
use super::super::WorldInspectionField;

/// Immutable inspector payload for one entity in one runtime generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldInspectionFieldsArtifact {
    generation: u64,
    entity: EntityId,
    fields: Arc<[WorldInspectionField]>,
}

impl WorldInspectionFieldsArtifact {
    pub(super) fn from_world(world: &World, entity: EntityId) -> Self {
        Self {
            generation: world.world_generation(),
            entity,
            fields: build_inspection_fields(world, entity).into(),
        }
    }

    pub(super) fn from_previous_generation(previous: &Self, generation: u64) -> Self {
        Self {
            generation,
            entity: previous.entity,
            fields: previous.fields.clone(),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn fields(&self) -> &[WorldInspectionField] {
        &self.fields
    }

    /// Computes changes for a single entity's inspector fields across generations.
    pub fn delta_from(&self, previous: &Self) -> WorldInspectionFieldDelta {
        let previous_fields = previous
            .fields
            .iter()
            .map(|field| {
                (
                    (
                        field.component_type_path.as_str(),
                        field.field_name.as_str(),
                    ),
                    field,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let current_fields = self
            .fields
            .iter()
            .map(|field| {
                (
                    (
                        field.component_type_path.as_str(),
                        field.field_name.as_str(),
                    ),
                    field,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let changed_fields = self
            .fields
            .iter()
            .filter(|field| {
                previous_fields
                    .get(&(
                        field.component_type_path.as_str(),
                        field.field_name.as_str(),
                    ))
                    .is_none_or(|previous_field| *previous_field != *field)
            })
            .cloned()
            .collect();
        let removed_fields = previous
            .fields
            .iter()
            .filter(|field| {
                !current_fields.contains_key(&(
                    field.component_type_path.as_str(),
                    field.field_name.as_str(),
                ))
            })
            .map(WorldInspectionFieldPath::from_field)
            .collect();

        WorldInspectionFieldDelta {
            previous_generation: previous.generation,
            generation: self.generation,
            entity: self.entity,
            changed_fields,
            removed_fields,
        }
    }
}

/// Stable identity for an inspector property in a runtime artifact.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldInspectionFieldPath {
    component_type_path: String,
    field_name: String,
}

impl WorldInspectionFieldPath {
    fn from_field(field: &WorldInspectionField) -> Self {
        Self {
            component_type_path: field.component_type_path.clone(),
            field_name: field.field_name.clone(),
        }
    }

    pub fn component_type_path(&self) -> &str {
        &self.component_type_path
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }
}

/// Entity/property-addressable inspector changes between focused-field artifacts.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldInspectionFieldDelta {
    previous_generation: u64,
    generation: u64,
    entity: EntityId,
    changed_fields: Vec<WorldInspectionField>,
    removed_fields: Vec<WorldInspectionFieldPath>,
}

impl WorldInspectionFieldDelta {
    pub const fn previous_generation(&self) -> u64 {
        self.previous_generation
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn changed_fields(&self) -> &[WorldInspectionField] {
        &self.changed_fields
    }

    pub fn removed_fields(&self) -> &[WorldInspectionFieldPath] {
        &self.removed_fields
    }
}
