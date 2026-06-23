#[cfg(test)]
mod lifecycle_failures;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::{
        backend::MockVmBackend, builtin_host_module_descriptors, module_descriptor,
        render_script_host_modules_markdown, write_script_host_modules_markdown,
        BuiltinVmBackendFamily, CapabilitySet, HostExportFunction, HostExportRegistry,
        HostRegistry, HotReloadCoordinator, PluginHostDriver, ScriptBridgeMethodBinding,
        ScriptBridgeMethodDescriptor, ScriptHostInterfaceMarkdownOptions, UnavailableVmBackend,
        VmBackend, VmBackendFamily, VmError, VmPluginHostContext, VmPluginInstance,
        VmPluginManager, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
        VmPluginSlotLifecycle, VmPluginSlotRecord, BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE,
        PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME,
        VM_PLUGIN_RUNTIME_NAME,
    };
    use crate::core::framework::bridge::PluginInterface;
    use crate::core::framework::script::{
        ScriptHostFieldDescriptor, ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor,
        ScriptHostParameterDescriptor, ScriptHostPrototypeKind, ScriptHostTypeDescriptor,
        ScriptHostTypeRef, ScriptHostValue, ScriptHostValueKind, ZirconScriptType,
    };
    use crate::core::{CoreRuntime, PluginContext};
    use crate::plugin::{
        PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginPackageManifest,
        RuntimeExtensionRegistry,
    };

    mod bridge_host;
    mod host_exports;
    mod module_surface;
    mod plugin_runtime;
    mod reflection_docs;
    mod support;
}
