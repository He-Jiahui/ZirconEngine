use crate::core::ModuleDescriptor;

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn apply_to_module(&mut self, mut descriptor: ModuleDescriptor) -> ModuleDescriptor {
        self.finalize();
        descriptor.managers.extend(self.managers().iter().cloned());
        descriptor
    }
}
