use crate::core::{CoreWeak, ModuleContext, PluginContext};

pub fn module_context(module_name: impl Into<String>, core: CoreWeak) -> ModuleContext {
    ModuleContext {
        module_name: module_name.into(),
        core,
    }
}

pub fn plugin_context(plugin_name: impl Into<String>, core: CoreWeak) -> PluginContext {
    PluginContext {
        plugin_name: plugin_name.into(),
        core,
        package_root: None,
        source_root: None,
        data_root: None,
    }
}
