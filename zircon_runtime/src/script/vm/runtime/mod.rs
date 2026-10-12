//! VM 插件运行时的公开协调器、管理器和槽位状态类型。

#[path = "hot_reload_coordinator.rs"]
mod hot_reload_coordinator;
#[path = "vm_plugin_manager.rs"]
mod vm_plugin_manager;
#[path = "vm_plugin_slot_record.rs"]
mod vm_plugin_slot_record;
#[path = "vm_plugin_slot_state.rs"]
mod vm_plugin_slot_state;

pub use hot_reload_coordinator::HotReloadCoordinator;
pub use vm_plugin_manager::VmPluginManager;
pub use vm_plugin_slot_record::VmPluginSlotRecord;
pub use vm_plugin_slot_state::VmPluginSlotState;
