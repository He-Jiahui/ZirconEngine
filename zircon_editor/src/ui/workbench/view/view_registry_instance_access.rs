//! 工作台视图注册表的实例读侧；实例数据是快照，位置一致性由宿主布局同步。
use super::{ViewInstance, ViewInstanceId, ViewRegistry};

impl ViewRegistry {
    pub fn instance(&self, instance_id: &ViewInstanceId) -> Option<&ViewInstance> {
        self.instances.get(instance_id)
    }

    pub fn instances(&self) -> Vec<ViewInstance> {
        self.instances.values().cloned().collect()
    }
}
