use super::{ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
    pub fn instance(&self, instance_id: &ViewInstanceId) -> Option<&ViewInstance> {
        self.instances.get(instance_id)
    }

    pub fn instances(&self) -> Vec<ViewInstance> {
        self.instances.values().cloned().collect()
    }
}
