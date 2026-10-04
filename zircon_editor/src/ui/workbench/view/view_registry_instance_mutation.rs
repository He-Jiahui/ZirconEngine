//! 实例退休和恢复序号维护，供关闭视图及项目工作区重新载入共用。
use super::{ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
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
#[path = "tests/view_registry_instance_mutation.rs"]
mod tests;
