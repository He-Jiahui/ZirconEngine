use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::ReflectFieldValue;

use crate::scene::components::NodeRecord;
use crate::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicComponent {
    pub type_path: String,
    #[serde(default)]
    pub plugin_owned: bool,
    #[serde(default)]
    pub fields: Vec<ReflectFieldValue>,
}

impl DynamicComponent {
    pub fn new(
        type_path: impl Into<String>,
        plugin_owned: bool,
        fields: Vec<ReflectFieldValue>,
    ) -> Self {
        Self {
            type_path: type_path.into(),
            plugin_owned,
            fields,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicResource {
    pub type_path: String,
    #[serde(default)]
    pub fields: Vec<ReflectFieldValue>,
}

impl DynamicResource {
    pub fn new(type_path: impl Into<String>, fields: Vec<ReflectFieldValue>) -> Self {
        Self {
            type_path: type_path.into(),
            fields,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicEntity {
    pub source_entity: EntityId,
    pub record: NodeRecord,
    #[serde(default)]
    pub components: Vec<DynamicComponent>,
}

impl DynamicEntity {
    pub fn new(
        source_entity: EntityId,
        record: NodeRecord,
        components: Vec<DynamicComponent>,
    ) -> Self {
        Self {
            source_entity,
            record,
            components,
        }
    }
}
