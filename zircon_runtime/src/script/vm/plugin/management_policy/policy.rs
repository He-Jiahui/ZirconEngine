use serde::{Deserialize, Serialize};

use super::{
    VmPluginGarbageCollectionPolicy, VmPluginHotReloadPolicy, VmPluginManagementPolicyResult,
    VmPluginMemoryPolicy,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginManagementPolicy {
    #[serde(default)]
    pub hot_reload: VmPluginHotReloadPolicy,
    #[serde(default)]
    pub garbage_collection: VmPluginGarbageCollectionPolicy,
    #[serde(default)]
    pub memory: VmPluginMemoryPolicy,
}

impl Default for VmPluginManagementPolicy {
    fn default() -> Self {
        Self {
            hot_reload: VmPluginHotReloadPolicy::PreserveState,
            garbage_collection: VmPluginGarbageCollectionPolicy::backend_managed(),
            memory: VmPluginMemoryPolicy::default(),
        }
    }
}

impl VmPluginManagementPolicy {
    pub fn with_hot_reload(mut self, hot_reload: VmPluginHotReloadPolicy) -> Self {
        self.hot_reload = hot_reload;
        self
    }

    pub fn with_garbage_collection(
        mut self,
        garbage_collection: VmPluginGarbageCollectionPolicy,
    ) -> Self {
        self.garbage_collection = garbage_collection;
        self
    }

    pub fn with_memory(mut self, memory: VmPluginMemoryPolicy) -> Self {
        self.memory = memory;
        self
    }

    pub fn validate(&self) -> VmPluginManagementPolicyResult<()> {
        self.garbage_collection.validate()?;
        self.memory.validate()
    }
}
