//! VM 宿主、插件生命周期和反射文档测试的共享夹具及模块入口。

#[cfg(test)]
#[path = "lifecycle_failures.rs"]
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
    HotReloadCoordinator, PluginHostDriver, ScriptBridgeMethodDescriptor,
    ScriptHostInterfaceMarkdownOptions, UnavailableVmBackend, VmBackend, VmBackendFamily, VmError,
    VmHostInterfaceError, VmHostInterfaceRegistry, VmInterfaceCaller, VmPluginHostContext,
    VmPluginInstance, VmPluginManager, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
    VmPluginSlotLifecycle, VmPluginSlotRecord, VmSystemStage, BRIDGE_HOST_CAPABILITY,
    BRIDGE_HOST_MODULE, PLUGIN_HOST_DRIVER_NAME, SCRIPT_MODULE_NAME, VM_BT_NODE_CAPABILITY,
    VM_EDITOR_OPERATION_CAPABILITY, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
    VM_RPC_HANDLER_CAPABILITY, VM_SYSTEM_CAPABILITY,
};
#[cfg(test)]
use crate::core::framework::bridge::PluginInterface;
#[cfg(test)]
use crate::core::framework::script::{
    ScriptHostArguments, ScriptHostError, ScriptHostFieldDescriptor, ScriptHostFunctionDescriptor,
    ScriptHostModuleDescriptor, ScriptHostOwnedArgumentSource, ScriptHostParameterDescriptor,
    ScriptHostPrototypeKind, ScriptHostTypeDescriptor, ScriptHostTypeRef, ScriptHostValue,
    ScriptHostValueKind, ScriptHostValueRef, ZirconScriptType,
};
#[cfg(test)]
use crate::core::{CoreRuntime, PluginContext};
#[cfg(test)]
use crate::plugin::RuntimeExtensionRegistry;

#[cfg(test)]
#[path = "bridge_host.rs"]
mod bridge_host;
#[cfg(test)]
#[path = "host_exports.rs"]
mod host_exports;
#[cfg(test)]
#[path = "host_interfaces.rs"]
mod host_interfaces;
#[cfg(test)]
#[path = "module_surface.rs"]
mod module_surface;
#[cfg(test)]
#[path = "plugin_runtime.rs"]
mod plugin_runtime;
#[cfg(test)]
#[path = "reflection_docs.rs"]
mod reflection_docs;
#[cfg(test)]
#[path = "support.rs"]
mod support;
