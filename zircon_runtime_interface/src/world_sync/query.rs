use std::collections::BTreeMap;

use crate::math::Transform;
use crate::reflect::ReflectedValue;
use serde::{Deserialize, Serialize};

/// Stable runtime-world entity identity used by the transport contract.
pub type EntityId = u64;

/// One reflected component value requested from a matching entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSelector {
    pub type_name: String,
}

impl ComponentSelector {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
        }
    }
}

/// Presence constraints applied before selected component values are read.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryFilter {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub with: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub without: Vec<String>,
}

/// Reflected-component projection against one runtime-world generation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentWorldQuery {
    #[serde(default)]
    pub filter: QueryFilter,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub select: Vec<ComponentSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_hint: Option<u64>,
}

/// Runtime-owned hierarchy projection against one inspection generation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldHierarchyQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_hint: Option<u64>,
}

/// Focused entity Inspector projection against one runtime-world generation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldInspectionFieldsQuery {
    pub entity: EntityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_hint: Option<u64>,
}

/// Exact local-transform read used to begin a world-qualified editor transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldTransformSnapshotQuery {
    pub entity: EntityId,
}

/// Transport-neutral world query with no invalid filter/projection combinations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorldQuery {
    Components(ComponentWorldQuery),
    Hierarchy(WorldHierarchyQuery),
    InspectionFields(WorldInspectionFieldsQuery),
    TransformSnapshot(WorldTransformSnapshotQuery),
}

impl Default for WorldQuery {
    fn default() -> Self {
        Self::Components(ComponentWorldQuery::default())
    }
}

impl WorldQuery {
    pub fn hierarchy(generation_hint: Option<u64>) -> Self {
        Self::Hierarchy(WorldHierarchyQuery { generation_hint })
    }

    pub fn inspection_fields(entity: EntityId, generation_hint: Option<u64>) -> Self {
        Self::InspectionFields(WorldInspectionFieldsQuery {
            entity,
            generation_hint,
        })
    }

    pub fn transform_snapshot(entity: EntityId) -> Self {
        Self::TransformSnapshot(WorldTransformSnapshotQuery { entity })
    }

    pub fn generation_hint(&self) -> Option<u64> {
        match self {
            Self::Components(query) => query.generation_hint,
            Self::Hierarchy(query) => query.generation_hint,
            Self::InspectionFields(query) => query.generation_hint,
            Self::TransformSnapshot(_) => None,
        }
    }

    pub fn with_generation_hint(mut self, generation_hint: Option<u64>) -> Self {
        match &mut self {
            Self::Components(query) => query.generation_hint = generation_hint,
            Self::Hierarchy(query) => query.generation_hint = generation_hint,
            Self::InspectionFields(query) => query.generation_hint = generation_hint,
            Self::TransformSnapshot(_) => {}
        }
        self
    }

    pub fn request_item_count(&self) -> usize {
        match self {
            Self::Components(query) => query
                .filter
                .with
                .len()
                .saturating_add(query.filter.without.len())
                .saturating_add(query.select.len())
                .saturating_add(1),
            Self::Hierarchy(_) | Self::InspectionFields(_) | Self::TransformSnapshot(_) => 1,
        }
    }

    /// Applies the authoritative component-generation short-circuit and canonical row order.
    pub fn component_result_for_generation(
        &self,
        generation: u64,
        mut rows: Vec<EntityRow>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::Components(_)));
        if generation != u64::MAX && self.generation_hint() == Some(generation) {
            WorldQueryResult::NotModified { generation }
        } else {
            rows.sort_by_key(|row| row.entity);
            WorldQueryResult::ComponentRows { generation, rows }
        }
    }

    /// Applies the authoritative hierarchy-generation short-circuit.
    pub fn hierarchy_result_for_generation(
        &self,
        generation: u64,
        rows: Vec<WorldHierarchyRow>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::Hierarchy(_)));
        if generation != u64::MAX && self.generation_hint() == Some(generation) {
            WorldQueryResult::NotModified { generation }
        } else {
            WorldQueryResult::HierarchyRows { generation, rows }
        }
    }

    /// Applies the focused Inspector short-circuit while preserving missing-entity state.
    pub fn inspection_fields_result_for_generation(
        &self,
        generation: u64,
        entity: EntityId,
        fields: Option<Vec<WorldInspectionFieldRow>>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::InspectionFields(_)));
        if generation != u64::MAX && self.generation_hint() == Some(generation) {
            WorldQueryResult::NotModified { generation }
        } else if let Some(fields) = fields {
            WorldQueryResult::InspectionFields {
                generation,
                entity,
                fields,
            }
        } else {
            WorldQueryResult::EntityMissing { generation, entity }
        }
    }

    pub fn transform_snapshot_result(
        &self,
        generation: u64,
        world_replacement_epoch: u64,
        entity: EntityId,
        transform: Option<Transform>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::TransformSnapshot(_)));
        match transform {
            Some(transform) => WorldQueryResult::TransformSnapshot {
                generation,
                world_replacement_epoch,
                entity,
                transform,
            },
            None => WorldQueryResult::EntityMissing { generation, entity },
        }
    }
}

/// One entity and the deterministically ordered component values selected for it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRow {
    pub entity: EntityId,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub components: BTreeMap<String, serde_json::Value>,
}

/// One runtime-owned hierarchy row shared by inspection and remote editor projections.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldHierarchyRow {
    pub entity: EntityId,
    pub parent: Option<EntityId>,
    pub depth: u32,
    pub display_name: String,
    pub kind: String,
    pub subtree_hash: u64,
    pub active_in_hierarchy: bool,
    pub has_children: bool,
}

/// One editor-visible reflected field in a focused runtime Inspector projection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldInspectionFieldRow {
    pub component_type_path: String,
    pub component_display_name: String,
    pub field_name: String,
    pub field_display_name: String,
    pub value_type_path: String,
    pub value: ReflectedValue,
    pub writable: bool,
    pub serializable: bool,
    pub plugin_owned: bool,
}

/// Generation-qualified query response or an explicit same-generation short circuit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorldQueryResult {
    ComponentRows {
        generation: u64,
        rows: Vec<EntityRow>,
    },
    HierarchyRows {
        generation: u64,
        rows: Vec<WorldHierarchyRow>,
    },
    InspectionFields {
        generation: u64,
        entity: EntityId,
        fields: Vec<WorldInspectionFieldRow>,
    },
    TransformSnapshot {
        generation: u64,
        world_replacement_epoch: u64,
        entity: EntityId,
        transform: Transform,
    },
    EntityMissing {
        generation: u64,
        entity: EntityId,
    },
    NotModified {
        generation: u64,
    },
}
