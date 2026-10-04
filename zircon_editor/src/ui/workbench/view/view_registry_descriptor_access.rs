//! 注册表读侧：按种类键查询，或按当前能力门槛投影可列举视图；列举不创建实例。
use super::{ViewDescriptor, ViewDescriptorId, ViewRegistry};

impl ViewRegistry {
    pub fn descriptor(&self, descriptor_id: &ViewDescriptorId) -> Option<&ViewDescriptor> {
        self.descriptors.get(descriptor_id)
    }

    /// 返回当前能力允许展示的描述符快照；HashMap次序无保证，菜单或持久化消费者应自行排序。
    pub fn list_descriptors(&self) -> Vec<ViewDescriptor> {
        let mut descriptors = Vec::with_capacity(self.descriptors.len());
        descriptors.extend(
            self.descriptors
                .values()
                .filter(|descriptor| self.descriptor_capability_error(descriptor).is_none())
                .cloned(),
        );
        descriptors
    }
}

#[cfg(test)]
#[path = "tests/view_registry_descriptor_access_optimization_tests.rs"]
mod optimization_tests;
