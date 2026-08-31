use std::collections::hash_map::Entry;

use super::{ViewDescriptor, ViewRegistry};

impl ViewRegistry {
    pub fn register_view(&mut self, descriptor: ViewDescriptor) -> Result<(), String> {
        match self.descriptors.entry(descriptor.descriptor_id.clone()) {
            Entry::Occupied(_) => Err(format!(
                "view descriptor {} already registered",
                descriptor.descriptor_id.0
            )),
            Entry::Vacant(entry) => {
                entry.insert(descriptor);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
#[path = "view_registry_register_view/entry_tests.rs"]
mod entry_tests;
