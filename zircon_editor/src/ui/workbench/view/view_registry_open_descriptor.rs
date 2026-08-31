use serde_json::Value;

use super::workbench_slot_to_view_host::workbench_slot_to_view_host;
use super::{ViewDescriptorId, ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
    pub fn open_descriptor(
        &mut self,
        descriptor_id: ViewDescriptorId,
    ) -> Result<ViewInstance, String> {
        let descriptor = self
            .descriptors
            .get(&descriptor_id)
            .ok_or_else(|| format!("missing view descriptor {}", descriptor_id.0))?;
        if let Some(error) = self.descriptor_capability_error(descriptor) {
            return Err(error);
        }

        let multi_instance = descriptor.multi_instance;
        if !multi_instance {
            if let Some(instance_id) = self.single_instance_index.get(&descriptor_id) {
                return self
                    .instances
                    .get(instance_id)
                    .cloned()
                    .ok_or_else(|| "single instance index is stale".to_string());
            }
        }

        let default_title = descriptor.default_title.clone();
        let workbench_slot = descriptor.workbench_slot;
        let counter = self.counters.entry(descriptor_id.clone()).or_insert(0);
        *counter += 1;
        let instance_id = ViewInstanceId::new(format!("{}#{}", descriptor_id.0, counter));
        let instance = ViewInstance {
            instance_id: instance_id.clone(),
            descriptor_id: descriptor_id.clone(),
            title: default_title,
            serializable_payload: Value::Null,
            dirty: false,
            host: workbench_slot_to_view_host(workbench_slot),
        };

        if !multi_instance {
            self.single_instance_index
                .insert(descriptor_id, instance_id.clone());
        }
        self.instances.insert(instance_id, instance.clone());
        Ok(instance)
    }
}

#[cfg(test)]
impl ViewRegistry {
    pub(crate) fn open_descriptor_with_retired_clone_for_benchmark(
        &self,
        descriptor_id: ViewDescriptorId,
    ) -> Result<ViewInstance, String> {
        let descriptor = self
            .descriptors
            .get(&descriptor_id)
            .cloned()
            .ok_or_else(|| format!("missing view descriptor {}", descriptor_id.0))?;
        if let Some(error) = self.descriptor_capability_error(&descriptor) {
            return Err(error);
        }
        debug_assert!(!descriptor.multi_instance);
        let instance_id = self
            .single_instance_index
            .get(&descriptor_id)
            .ok_or_else(|| "single instance index is missing".to_string())?;
        self.instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| "single instance index is stale".to_string())
    }
}
