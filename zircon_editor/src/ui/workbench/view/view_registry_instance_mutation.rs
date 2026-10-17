//! 视图实例的打开、恢复与退休共用身份表、单例索引及序号，集中维护三者的一致性。
use serde_json::Value;

use super::workbench_slot_to_view_host::workbench_slot_to_view_host;
use super::{ViewDescriptorId, ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
    /// 单实例种类复用现有实例，多实例种类分配新键；descriptor须已注册且满足当前能力。
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
        let instance_id = loop {
            *counter = counter.checked_add(1).ok_or_else(|| {
                format!("view instance counter exhausted for {}", descriptor_id.0)
            })?;
            let candidate = ViewInstanceId::new(format!("{}#{}", descriptor_id.0, counter));
            if !self.instances.contains_key(&candidate) {
                break candidate;
            }
        };
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

    /// 只接受已注册且能力满足的种类；恢复身份还必须与既有实例和单例索引保持一致。
    pub fn restore_instance(&mut self, instance: ViewInstance) -> Result<ViewInstance, String> {
        let Some(descriptor) = self.descriptors.get(&instance.descriptor_id) else {
            return Err(format!(
                "cannot restore missing descriptor {}",
                instance.descriptor_id.0
            ));
        };
        if let Some(error) = self.descriptor_capability_error(descriptor) {
            return Err(error);
        }
        let multi_instance = descriptor.multi_instance;
        // 恢复不得改变已有实例或单例索引的身份归属。
        if self
            .instances
            .get(&instance.instance_id)
            .is_some_and(|existing| existing.descriptor_id != instance.descriptor_id)
        {
            return Err(format!(
                "view instance {} already belongs to another descriptor",
                instance.instance_id.0
            ));
        }
        if !multi_instance
            && self
                .single_instance_index
                .get(&instance.descriptor_id)
                .is_some_and(|existing| existing != &instance.instance_id)
        {
            return Err(format!(
                "view descriptor {} already has another single instance",
                instance.descriptor_id.0
            ));
        }
        self.update_counter(&instance);
        if !multi_instance {
            self.single_instance_index
                .insert(instance.descriptor_id.clone(), instance.instance_id.clone());
        }
        self.instances
            .insert(instance.instance_id.clone(), instance.clone());
        Ok(instance)
    }

    /// 关闭具体pane并在它正是单实例索引目标时撤销该索引；布局/焦点仍由宿主同步。
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

    /// 项目工作区切换时重置具体实例身份和计数，随后由恢复链导入项目快照。
    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.single_instance_index.clear();
        self.counters.clear();
    }

    // 恢复实例后维持后续自动编号不与现有数字后缀相撞；此处不验证ID前缀，调用方须保证身份归属。
    pub(super) fn update_counter(&mut self, instance: &ViewInstance) {
        let Some((_, suffix)) = instance.instance_id.0.rsplit_once('#') else {
            return;
        };
        let Ok(value) = suffix.parse::<usize>() else {
            return;
        };
        if let Some(counter) = self.counters.get_mut(&instance.descriptor_id) {
            *counter = (*counter).max(value);
            return;
        }
        self.counters.insert(instance.descriptor_id.clone(), value);
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

#[cfg(test)]
#[path = "tests/view_registry_instance_mutation.rs"]
mod tests;
