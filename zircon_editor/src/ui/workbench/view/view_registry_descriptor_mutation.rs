use super::{ViewDescriptor, ViewDescriptorId, ViewRegistry};

impl ViewRegistry {
    /// Removes a descriptor after all of its live instances have been retired.
    /// 扩展撤销视图种类前必须退休所有活实例；失败时保留声明和索引供宿主处理。
    pub fn unregister_view(
        &mut self,
        descriptor_id: &ViewDescriptorId,
    ) -> Result<ViewDescriptor, String> {
        if self
            .instances
            .values()
            .any(|instance| &instance.descriptor_id == descriptor_id)
        {
            return Err(format!(
                "cannot unregister view descriptor {} while instances are open",
                descriptor_id.0
            ));
        }
        let removed = self.descriptors.remove(descriptor_id).ok_or_else(|| {
            format!(
                "cannot unregister missing view descriptor {}",
                descriptor_id.0
            )
        })?;
        self.single_instance_index.remove(descriptor_id);
        self.counters.remove(descriptor_id);
        Ok(removed)
    }
}

#[cfg(test)]
#[path = "tests/view_registry_descriptor_mutation.rs"]
mod tests;
