//! Script host、插件驱动器和 VM 管理器在模块注册时共享的稳定名称。

pub const SCRIPT_MODULE_NAME: &str = "ScriptModule";
pub const PLUGIN_HOST_DRIVER_NAME: &str = "ScriptModule.Driver.PluginHostDriver";
pub const VM_PLUGIN_RUNTIME_NAME: &str = "ScriptModule.Plugin.VmPluginRuntime";
pub const VM_PLUGIN_MANAGER_NAME: &str = "ScriptModule.Manager.VmPluginManager";
