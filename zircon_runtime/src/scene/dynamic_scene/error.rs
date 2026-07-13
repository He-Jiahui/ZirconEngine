use std::sync::Arc;

use thiserror::Error;
use zircon_runtime_interface::reflect::ReflectError;
use zircon_runtime_interface::serialization::{LoadError, WriteError};

use crate::scene::{EntityId, SceneError};

#[derive(Clone, Debug, Error)]
pub enum DynamicSceneError {
    #[error("unsupported dynamic scene schema {actual}; expected {expected}")]
    UnsupportedSchema { expected: String, actual: String },
    #[error("unsupported dynamic scene format version {actual}; expected {expected}")]
    UnsupportedFormatVersion { expected: u32, actual: u32 },
    #[error("dynamic scene contains duplicate source entity {entity}")]
    DuplicateSourceEntity { entity: EntityId },
    #[error("dynamic scene contains duplicate component type descriptor `{type_id}`")]
    DuplicateComponentTypeDescriptor { type_id: String },
    #[error("dynamic scene component type descriptor `{type_id}` is invalid: {reason}")]
    InvalidComponentTypeDescriptor { type_id: String, reason: String },
    #[error("dynamic scene component type descriptor `{type_id}` conflicts with target world")]
    ComponentTypeDescriptorConflict { type_id: String },
    #[error("dynamic scene entity {entity} references missing parent {parent}")]
    MissingSceneParent { entity: EntityId, parent: EntityId },
    #[error("no free target entity id remains while remapping source entity {source_entity}")]
    EntityIdSpaceExhausted { source_entity: EntityId },
    #[error("world mutation failed: {0}")]
    WorldMutation(#[from] SceneError),
    #[error("dynamic scene parse failed: {reason}")]
    Parse { reason: String },
    #[error("dynamic scene scene-asset conversion failed: {reason}")]
    SceneAsset { reason: String },
    #[error("dynamic scene I/O failed: {reason}")]
    Io { reason: String },
    #[error("dynamic scene spawn task `{label}` result is unavailable")]
    SpawnTaskResultUnavailable { label: String },
    #[error("unsupported reflected value `{type_name}` for `{context}`")]
    UnsupportedValue {
        context: String,
        type_name: &'static str,
    },
    #[error(transparent)]
    Reflect(#[from] ReflectError),
    #[error(transparent)]
    SerializationLoad(Arc<LoadError>),
    #[error(transparent)]
    SerializationWrite(Arc<WriteError>),
}

impl From<LoadError> for DynamicSceneError {
    fn from(error: LoadError) -> Self {
        Self::SerializationLoad(Arc::new(error))
    }
}

impl From<WriteError> for DynamicSceneError {
    fn from(error: WriteError) -> Self {
        Self::SerializationWrite(Arc::new(error))
    }
}

impl PartialEq for DynamicSceneError {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
            && self.to_string() == other.to_string()
    }
}

impl Eq for DynamicSceneError {}
