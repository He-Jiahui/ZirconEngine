use crate::core::{ManagerDescriptor, ModuleDescriptor};

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn managers(&self) -> &[ManagerDescriptor] {
        self.managers.values()
    }

    pub fn modules(&self) -> &[ModuleDescriptor] {
        self.modules.values()
    }
}
