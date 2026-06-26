#[cfg(test)]
mod lifecycle_failures;

#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::{Arc, Mutex};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use super::{
    backend::MockVmBackend, builtin_host_module_descriptors, module_descriptor,
    render_script_host_modules_markdown, write_script_host_modules_markdown,
    BuiltinVmBackendFamily, CapabilitySet, HostExportFunction, HostExportRegistry, HostRegistry,
    HotReloadCoordinator, PluginHostDriver, ScriptBridgeMethodBinding,
    ScriptBridgeMethodDescriptor, ScriptHostInterfaceMarkdownOptions, UnavailableVmBackend,
    VmBackend, VmBackendFamily, VmError, VmPluginHostContext, VmPluginInstance, VmPluginManager,
    VmPluginManifest, VmPluginPackage, VmPluginPackageSource, VmPluginSlotLifecycle,
    VmPluginSlotRecord, BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE, PLUGIN_HOST_DRIVER_NAME,
    SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
};
#[cfg(test)]
use crate::core::framework::bridge::PluginInterface;
#[cfg(test)]
use crate::core::framework::script::{
    ScriptHostFieldDescriptor, ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor,
    ScriptHostParameterDescriptor, ScriptHostPrototypeKind, ScriptHostTypeDescriptor,
    ScriptHostTypeRef, ScriptHostValue, ScriptHostValueKind, ZirconScriptType,
};
#[cfg(test)]
use crate::core::{CoreRuntime, PluginContext};
#[cfg(test)]
use crate::plugin::{
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginPackageManifest,
    RuntimeExtensionRegistry,
};

#[cfg(test)]
mod bridge_host;
#[cfg(test)]
mod host_exports;
#[cfg(test)]
mod module_surface;
#[cfg(test)]
mod plugin_runtime;
#[cfg(test)]
mod reflection_docs;
#[cfg(test)]
mod support;
