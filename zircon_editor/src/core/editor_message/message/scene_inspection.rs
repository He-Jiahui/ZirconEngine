use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

/// Stable inspector-property identity carried by an inspection notification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionPropertyPath {
    component_type_path: String,
    field_name: String,
}

impl SceneInspectionPropertyPath {
    pub fn new(component_type_path: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            component_type_path: component_type_path.into(),
            field_name: field_name.into(),
        }
    }

    pub fn component_type_path(&self) -> &str {
        &self.component_type_path
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }
}

/// Focused-inspector change identities; field values stay in the runtime artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionFieldsDelta {
    entity: Option<EntityId>,
    requires_resync: bool,
    changed_properties: Vec<SceneInspectionPropertyPath>,
    removed_properties: Vec<SceneInspectionPropertyPath>,
}

impl SceneInspectionFieldsDelta {
    pub fn unchanged(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: false,
            changed_properties: Vec::new(),
            removed_properties: Vec::new(),
        }
    }

    pub fn delta(
        entity: EntityId,
        changed_properties: Vec<SceneInspectionPropertyPath>,
        removed_properties: Vec<SceneInspectionPropertyPath>,
    ) -> Self {
        Self {
            entity: Some(entity),
            requires_resync: false,
            changed_properties,
            removed_properties,
        }
    }

    /// Selection changed or the consumer fell behind, so it must read the focused artifact again.
    pub fn resync(entity: Option<EntityId>) -> Self {
        Self {
            entity,
            requires_resync: true,
            changed_properties: Vec::new(),
            removed_properties: Vec::new(),
        }
    }

    pub const fn entity(&self) -> Option<EntityId> {
        self.entity
    }

    pub const fn requires_resync(&self) -> bool {
        self.requires_resync
    }

    pub fn changed_properties(&self) -> &[SceneInspectionPropertyPath] {
        &self.changed_properties
    }

    pub fn removed_properties(&self) -> &[SceneInspectionPropertyPath] {
        &self.removed_properties
    }
}

/// Runtime-scene change notification without a copied hierarchy or inspector snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionMessage {
    previous_generation: Option<u64>,
    generation: u64,
    focused_entity: Option<EntityId>,
    added_entities: Vec<EntityId>,
    changed_entities: Vec<EntityId>,
    removed_entities: Vec<EntityId>,
    focused_fields: SceneInspectionFieldsDelta,
}

impl SceneInspectionMessage {
    pub fn delta(
        previous_generation: u64,
        generation: u64,
        focused_entity: Option<EntityId>,
        added_entities: Vec<EntityId>,
        changed_entities: Vec<EntityId>,
        removed_entities: Vec<EntityId>,
        focused_fields: SceneInspectionFieldsDelta,
    ) -> Self {
        Self {
            previous_generation: Some(previous_generation),
            generation,
            focused_entity,
            added_entities,
            changed_entities,
            removed_entities,
            focused_fields,
        }
    }

    /// The receiver has no compatible base generation and must read the runtime artifact anew.
    pub fn resync(generation: u64, focused_entity: Option<EntityId>) -> Self {
        Self {
            previous_generation: None,
            generation,
            focused_entity,
            added_entities: Vec::new(),
            changed_entities: Vec::new(),
            removed_entities: Vec::new(),
            focused_fields: SceneInspectionFieldsDelta::resync(focused_entity),
        }
    }

    pub const fn previous_generation(&self) -> Option<u64> {
        self.previous_generation
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn focused_entity(&self) -> Option<EntityId> {
        self.focused_entity
    }

    pub fn added_entities(&self) -> &[EntityId] {
        &self.added_entities
    }

    pub fn changed_entities(&self) -> &[EntityId] {
        &self.changed_entities
    }

    pub fn removed_entities(&self) -> &[EntityId] {
        &self.removed_entities
    }

    pub fn focused_fields(&self) -> &SceneInspectionFieldsDelta {
        &self.focused_fields
    }

    pub const fn requires_resync(&self) -> bool {
        self.previous_generation.is_none()
    }
}
