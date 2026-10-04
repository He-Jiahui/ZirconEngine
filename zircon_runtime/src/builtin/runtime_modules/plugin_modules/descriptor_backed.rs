use std::sync::Arc;

use crate::core::ModuleDescriptor;
use crate::engine_module::EngineModule;

#[derive(Debug)]
struct DescriptorBackedEngineModule {
    descriptor: ModuleDescriptor,
}

impl EngineModule for DescriptorBackedEngineModule {
    fn module_name(&self) -> &str {
        &self.descriptor.name
    }

    fn module_description(&self) -> &str {
        &self.descriptor.description
    }

    fn descriptor(&self) -> ModuleDescriptor {
        self.descriptor.clone()
    }
}

pub(in crate::builtin::runtime_modules) fn descriptor_backed_module(
    descriptor: ModuleDescriptor,
) -> Arc<dyn EngineModule> {
    Arc::new(DescriptorBackedEngineModule { descriptor })
}

#[cfg(test)]
#[path = "tests/descriptor_backed.rs"]
mod tests;
