use std::sync::Arc;

use super::super::{PluginSlotId, VmError, VmPluginPackage, VmPluginSlotRecord};

/// 宿主脚本入口通过此门面请求插件装载、热重载和卸载；实际状态由管理器与协调器持有，门面不能绕过槽位生命周期。
pub trait VmPluginSlotLifecycle: Send + Sync {
    fn load_package(
        &self,
        backend_selector: &str,
        package: VmPluginPackage,
    ) -> Result<PluginSlotId, VmError>;

    fn hot_reload_slot(&self, slot: PluginSlotId, package: VmPluginPackage) -> Result<(), VmError>;

    fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError>;

    fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError>;

    fn list_slots(&self) -> Vec<VmPluginSlotRecord>;
}

impl<T> VmPluginSlotLifecycle for Arc<T>
where
    T: VmPluginSlotLifecycle + ?Sized,
{
    fn load_package(
        &self,
        backend_selector: &str,
        package: VmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        (**self).load_package(backend_selector, package)
    }

    fn hot_reload_slot(&self, slot: PluginSlotId, package: VmPluginPackage) -> Result<(), VmError> {
        (**self).hot_reload_slot(slot, package)
    }

    fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError> {
        (**self).unload_slot(slot)
    }

    fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        (**self).slot(slot)
    }

    fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        (**self).list_slots()
    }
}
