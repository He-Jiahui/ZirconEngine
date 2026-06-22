use thiserror::Error;
use zircon_runtime_interface::reflect::ReflectError;

use crate::scene::{ecs::StorageError, EntityId};

pub type SceneResult<T> = std::result::Result<T, SceneError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SceneError {
    #[error("cannot {operation} missing entity {entity}")]
    MissingEntity {
        operation: &'static str,
        entity: EntityId,
    },
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Reflect(#[from] ReflectError),
    #[error("component type {type_id} must be prefixed by plugin id {plugin_id}")]
    ComponentTypePluginPrefixMismatch { type_id: String, plugin_id: String },
    #[error("component type {type_id} already registered")]
    DuplicateComponentType { type_id: String },
    #[error("dynamic component type `{component_id}` is not registered")]
    UnregisteredDynamicComponentType { component_id: String },
    #[error("plugin `{plugin_id}` cannot unload while dynamic components are active: {active_components}")]
    PluginComponentsActive {
        plugin_id: String,
        active_components: String,
    },
    #[error("unknown property `{property_path}`")]
    UnknownDynamicComponentProperty { property_path: String },
    #[error("property `{property_path}` cannot be written to a dynamic component")]
    DynamicComponentPropertyUnsupportedValue { property_path: String },
    #[error("dynamic component `{component_id}` is not an object")]
    DynamicComponentNotObject { component_id: String },
    #[error("dynamic component type `{component_id}` does not declare property `{property}`")]
    UndeclaredDynamicComponentProperty {
        component_id: String,
        property: String,
    },
    #[error("dynamic component property `{component_id}.{property}` is not editable")]
    NonEditableDynamicComponentProperty {
        component_id: String,
        property: String,
    },
    #[error("{0}")]
    Message(String),
}

impl SceneError {
    pub(crate) fn missing_entity(operation: &'static str, entity: EntityId) -> Self {
        Self::MissingEntity { operation, entity }
    }
}

impl From<String> for SceneError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}
