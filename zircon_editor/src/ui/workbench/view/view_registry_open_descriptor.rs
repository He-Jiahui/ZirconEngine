use serde_json::Value;

use super::preferred_host_to_view_host::preferred_host_to_view_host;
use super::{ViewDescriptorId, ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
    pub fn open_descriptor(
        &mut self,
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

        if !descriptor.multi_instance {
            if let Some(instance_id) = self.single_instance_index.get(&descriptor_id) {
                return self
                    .instances
                    .get(instance_id)
                    .cloned()
                    .ok_or_else(|| "single instance index is stale".to_string());
            }
        }

        let counter = self.counters.entry(descriptor_id.clone()).or_insert(0);
        *counter += 1;
        let instance_id = ViewInstanceId::new(format!("{}#{}", descriptor_id.0, counter));
        let instance = ViewInstance {
            instance_id: instance_id.clone(),
            descriptor_id: descriptor_id.clone(),
            title: descriptor.default_title.clone(),
            serializable_payload: Value::Null,
            dirty: false,
            host: preferred_host_to_view_host(descriptor.preferred_host),
        };

        if !descriptor.multi_instance {
            self.single_instance_index
                .insert(descriptor_id, instance_id.clone());
        }
        self.instances.insert(instance_id, instance.clone());
        Ok(instance)
    }
}
