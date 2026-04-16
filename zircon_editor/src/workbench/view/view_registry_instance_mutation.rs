use super::{ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
    pub fn remove_instance(&mut self, instance_id: &ViewInstanceId) -> Option<ViewInstance> {
        let removed = self.instances.remove(instance_id)?;
        if self
            .single_instance_index
            .get(&removed.descriptor_id)
            .is_some_and(|current| current == instance_id)
        {
            self.single_instance_index.remove(&removed.descriptor_id);
        }
        Some(removed)
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.single_instance_index.clear();
        self.counters.clear();
    }

    pub(super) fn update_counter(&mut self, instance: &ViewInstance) {
        let Some((_, suffix)) = instance.instance_id.0.rsplit_once('#') else {
            return;
        };
        let Ok(value) = suffix.parse::<usize>() else {
            return;
        };
        let counter = self
            .counters
            .entry(instance.descriptor_id.clone())
            .or_insert(0);
        *counter = (*counter).max(value);
    }
}
