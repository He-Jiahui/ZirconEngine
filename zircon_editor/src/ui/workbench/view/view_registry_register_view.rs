use super::{ViewDescriptor, ViewRegistry};

impl ViewRegistry {
    pub fn register_view(&mut self, descriptor: ViewDescriptor) -> Result<(), String> {
        if self.descriptors.contains_key(&descriptor.descriptor_id) {
            return Err(format!(
                "view descriptor {} already registered",
                descriptor.descriptor_id.0
            ));
        }
        self.descriptors
            .insert(descriptor.descriptor_id.clone(), descriptor);
        Ok(())
    }
}
