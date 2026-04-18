use super::{ViewInstance, ViewRegistry};

impl ViewRegistry {
    pub fn restore_instance(&mut self, instance: ViewInstance) -> Result<ViewInstance, String> {
        let Some(multi_instance) = self
            .descriptors
            .get(&instance.descriptor_id)
            .map(|descriptor| descriptor.multi_instance)
        else {
            return Err(format!(
                "cannot restore missing descriptor {}",
                instance.descriptor_id.0
            ));
        };
        self.update_counter(&instance);
        if !multi_instance {
            self.single_instance_index
                .insert(instance.descriptor_id.clone(), instance.instance_id.clone());
        }
        self.instances
            .insert(instance.instance_id.clone(), instance.clone());
        Ok(instance)
    }
}
