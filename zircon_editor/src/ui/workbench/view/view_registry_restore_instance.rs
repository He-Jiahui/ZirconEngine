use super::{ViewInstance, ViewRegistry};

impl ViewRegistry {
    pub fn restore_instance(&mut self, instance: ViewInstance) -> Result<ViewInstance, String> {
        let Some(descriptor) = self.descriptors.get(&instance.descriptor_id).cloned() else {
            return Err(format!(
                "cannot restore missing descriptor {}",
                instance.descriptor_id.0
            ));
        };
        if let Some(error) = self.descriptor_capability_error(&descriptor) {
            return Err(error);
        }
        self.update_counter(&instance);
        if !descriptor.multi_instance {
            self.single_instance_index
                .insert(instance.descriptor_id.clone(), instance.instance_id.clone());
        }
        self.instances
            .insert(instance.instance_id.clone(), instance.clone());
        Ok(instance)
    }
}
