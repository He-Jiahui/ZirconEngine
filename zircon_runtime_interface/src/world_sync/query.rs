use std::collections::BTreeMap;

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

/// Transport-neutral query shared by in-process and serialized gateways.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldQuery {
    #[serde(default)]
    pub filter: QueryFilter,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub select: Vec<ComponentSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_hint: Option<u64>,
}

impl WorldQuery {
    /// Applies the one authoritative generation-hint short-circuit rule.
    pub fn result_for_generation(&self, generation: u64, rows: Vec<EntityRow>) -> WorldQueryResult {
        if self.generation_hint == Some(generation) {
            WorldQueryResult::NotModified { generation }
        } else {
            WorldQueryResult::Rows(rows)
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

/// Query response or an explicit same-generation short circuit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorldQueryResult {
    Rows(Vec<EntityRow>),
    NotModified { generation: u64 },
}
