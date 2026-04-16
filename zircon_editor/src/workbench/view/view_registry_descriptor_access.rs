use super::{ViewDescriptor, ViewDescriptorId, ViewRegistry};

impl ViewRegistry {
    pub fn descriptor(&self, descriptor_id: &ViewDescriptorId) -> Option<&ViewDescriptor> {
        self.descriptors.get(descriptor_id)
    }

    pub fn list_descriptors(&self) -> Vec<ViewDescriptor> {
        self.descriptors.values().cloned().collect()
    }
}
