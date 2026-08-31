use super::{ViewDescriptor, ViewDescriptorId, ViewRegistry};

impl ViewRegistry {
    /// Removes a descriptor after all of its live instances have been retired.
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
mod tests {
    use super::*;
    use crate::ui::workbench::view::ViewKind;

    #[test]
    fn unregister_view_removes_an_unreferenced_descriptor() {
        let mut registry = ViewRegistry::default();
        let descriptor = ViewDescriptor::new(
            ViewDescriptorId::new("plugin.example.panel"),
            ViewKind::ActivityView,
            "Panel",
        );
        registry.register_view(descriptor.clone()).unwrap();

        assert_eq!(
            registry.unregister_view(&descriptor.descriptor_id),
            Ok(descriptor)
        );
        assert!(registry
            .descriptor(&ViewDescriptorId::new("plugin.example.panel"))
            .is_none());
    }

    #[test]
    fn unregister_view_rejects_a_live_instance() {
        let mut registry = ViewRegistry::default();
        let descriptor = ViewDescriptor::new(
            ViewDescriptorId::new("plugin.example.panel"),
            ViewKind::ActivityView,
            "Panel",
        );
        registry.register_view(descriptor.clone()).unwrap();
        registry
            .open_descriptor(descriptor.descriptor_id.clone())
            .unwrap();

        let error = registry
            .unregister_view(&descriptor.descriptor_id)
            .expect_err("live instances must keep their descriptor alive");
        assert!(error.contains("instances are open"));
    }
}
