use serde::{Deserialize, Serialize};

use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator};

pub trait Resource: Send + Sync {
    fn id(&self) -> ResourceId;
    fn kind(&self) -> ResourceKind;
    fn primary_locator(&self) -> &ResourceLocator;
    fn revision(&self) -> u64;
    fn runtime_state(&self) -> RuntimeResourceState;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeResourceState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
    Error,
    Reloading,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceInspectorAdapterKey(String);

impl ResourceInspectorAdapterKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceTypeDescriptor {
    pub kind: ResourceKind,
    pub inspector_adapter: ResourceInspectorAdapterKey,
    pub extensions: Vec<String>,
}

impl ResourceTypeDescriptor {
    pub fn new(
        kind: ResourceKind,
        inspector_adapter: ResourceInspectorAdapterKey,
        extensions: Vec<String>,
    ) -> Self {
        Self {
            kind,
            inspector_adapter,
            extensions,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRuntimeInfo {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub primary_locator: ResourceLocator,
    pub revision: u64,
    pub runtime_state: RuntimeResourceState,
}

impl Resource for ResourceRuntimeInfo {
    fn id(&self) -> ResourceId {
        self.id
    }

    fn kind(&self) -> ResourceKind {
        self.kind
    }

    fn primary_locator(&self) -> &ResourceLocator {
        &self.primary_locator
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn runtime_state(&self) -> RuntimeResourceState {
        self.runtime_state
    }
}
