#[path = "bridge_host_module.rs"]
mod bridge_host_module;
#[path = "builtin_host_modules.rs"]
mod builtin_host_modules;
#[path = "constants.rs"]
mod constants;
#[path = "host_export_registry.rs"]
mod host_export_registry;
#[path = "host_registry.rs"]
mod host_registry;
#[path = "plugin_host_driver.rs"]
mod plugin_host_driver;
#[path = "reflection_docs/mod.rs"]
mod reflection_docs;
#[path = "script_call_table.rs"]
mod script_call_table;
#[path = "vm_plugin_host_context.rs"]
mod vm_plugin_host_context;
#[path = "vm_plugin_slot_lifecycle.rs"]
mod vm_plugin_slot_lifecycle;

pub use bridge_host_module::{
    register_bridge_host_module, ScriptBridgeCall, ScriptBridgeMethodDescriptor,
    BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE,
};
pub use builtin_host_modules::{builtin_host_capabilities, register_builtin_host_modules};
pub use constants::{
    PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};
pub use host_export_registry::{
    HostExportCallback, HostExportFunction, HostExportModuleRecord, HostExportRegistry,
};
pub use host_registry::{HostCapabilityRecord, HostRegistry, HostRegistryError};
pub use plugin_host_driver::PluginHostDriver;
pub use reflection_docs::{
    builtin_host_module_descriptors, render_script_host_modules_markdown,
    write_script_host_modules_markdown, ScriptHostInterfaceMarkdownOptions,
};
pub use script_call_table::{ScriptCallSite, ScriptCallSiteId, ScriptCallTable};
pub use vm_plugin_host_context::{VmPluginHostContext, VmReflectionSchemaInstaller};
pub use vm_plugin_slot_lifecycle::VmPluginSlotLifecycle;
