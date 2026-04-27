use crate::core::ModuleDescriptor;

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn apply_to_module(&self, mut descriptor: ModuleDescriptor) -> ModuleDescriptor {
        descriptor.managers.extend(self.managers.iter().cloned());
        descriptor
    }
}
