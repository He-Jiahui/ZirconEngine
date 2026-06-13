use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::framework::script::{
    ScriptHostError, ScriptHostFunctionDescriptor, ScriptHostParameterDescriptor, ScriptHostResult,
    ScriptHostValue, ScriptHostValueKind,
};
use crate::plugin::{
    BridgeInterfaceStatus, FrozenBridgeTable, InterfaceSlot, PluginInterfaceMethodManifest,
    PluginPackageManifest, RuntimeExtensionRegistryError,
};

use super::super::VmError;
use super::{HostExportFunction, HostExportRegistry};

pub const BRIDGE_HOST_MODULE: &str = "zr.zircon.bridge";
pub const BRIDGE_HOST_CAPABILITY: &str = "bridge.call";
const BRIDGE_HOST_MODULE_VERSION: &str = "0.1.0";

pub type ScriptBridgeMethodFn =
    Arc<dyn Fn(ScriptBridgeCall) -> ScriptHostResult + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ScriptBridgeCall {
    pub interface_slot: InterfaceSlot,
    pub method_slot: u32,
    pub arguments: Vec<ScriptHostValue>,
}

#[derive(Clone)]
pub struct ScriptBridgeMethodDescriptor {
    function_name: String,
    interface_id: String,
    method_slot: u32,
    return_value_kind: ScriptHostValueKind,
    parameters: Vec<ScriptHostParameterDescriptor>,
    required_capabilities: Vec<String>,
    documentation: Option<String>,
    method: ScriptBridgeMethodFn,
}

impl ScriptBridgeMethodDescriptor {
    pub fn new<F>(
        function_name: impl Into<String>,
        interface_id: impl Into<String>,
        method_slot: u32,
        return_value_kind: ScriptHostValueKind,
        method: F,
    ) -> Self
    where
        F: Fn(ScriptBridgeCall) -> ScriptHostResult + Send + Sync + 'static,
    {
        Self {
            function_name: function_name.into(),
            interface_id: interface_id.into(),
            method_slot,
            return_value_kind,
            parameters: Vec::new(),
            required_capabilities: vec![BRIDGE_HOST_CAPABILITY.to_string()],
            documentation: None,
            method: Arc::new(method),
        }
    }

    pub fn with_parameter(mut self, parameter: ScriptHostParameterDescriptor) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }

    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub fn interface_id(&self) -> &str {
        &self.interface_id
    }

    pub const fn method_slot(&self) -> u32 {
        self.method_slot
    }

    pub fn from_manifest_method(
        interface_id: &str,
        method: &PluginInterfaceMethodManifest,
        bridge_method: ScriptBridgeMethodFn,
    ) -> Self {
        let mut required_capabilities = vec![BRIDGE_HOST_CAPABILITY.to_string()];
        required_capabilities.extend(method.required_capabilities.iter().cloned());
        required_capabilities.sort();
        required_capabilities.dedup();
        Self {
            function_name: method.name.clone(),
            interface_id: interface_id.to_string(),
            method_slot: method.method_slot,
            return_value_kind: method.return_value_kind,
            parameters: method.parameters.clone(),
            required_capabilities,
            documentation: method.documentation.clone(),
            method: bridge_method,
        }
    }
}

#[derive(Clone)]
pub struct ScriptBridgeMethodBinding {
    interface_id: String,
    method_name: String,
    method: ScriptBridgeMethodFn,
}

impl ScriptBridgeMethodBinding {
    pub fn new<F>(
        interface_id: impl Into<String>,
        method_name: impl Into<String>,
        method: F,
    ) -> Self
    where
        F: Fn(ScriptBridgeCall) -> ScriptHostResult + Send + Sync + 'static,
    {
        Self {
            interface_id: interface_id.into(),
            method_name: method_name.into(),
            method: Arc::new(method),
        }
    }
}

pub fn script_bridge_method_descriptors_from_manifest(
    manifest: &PluginPackageManifest,
    bindings: impl IntoIterator<Item = ScriptBridgeMethodBinding>,
) -> Result<Vec<ScriptBridgeMethodDescriptor>, VmError> {
    let mut bindings_by_method = BTreeMap::new();
    for binding in bindings {
        let key = (binding.interface_id, binding.method_name);
        if bindings_by_method
            .insert(key.clone(), binding.method)
            .is_some()
        {
            return Err(VmError::Operation(format!(
                "duplicate script bridge method binding `{}.{}`",
                key.0, key.1
            )));
        }
    }

    let mut descriptors = Vec::new();
    for (interface, method) in manifest.bridge_methods() {
        let key = (interface.id.clone(), method.name.clone());
        let Some(bridge_method) = bindings_by_method.remove(&key) else {
            return Err(VmError::Operation(format!(
                "script bridge method `{}.{}` is declared but has no binding",
                key.0, key.1
            )));
        };
        descriptors.push(ScriptBridgeMethodDescriptor::from_manifest_method(
            &interface.id,
            method,
            bridge_method,
        ));
    }

    if let Some(((interface_id, method_name), _)) = bindings_by_method.into_iter().next() {
        return Err(VmError::Operation(format!(
            "script bridge method binding `{interface_id}.{method_name}` is not declared by the package manifest"
        )));
    }

    Ok(descriptors)
}

pub fn register_bridge_host_module(
    exports: &HostExportRegistry,
    bridge_table: FrozenBridgeTable,
    methods: impl IntoIterator<Item = ScriptBridgeMethodDescriptor>,
) -> Result<(), VmError> {
    let methods = methods.into_iter().collect::<Vec<_>>();
    let mut descriptor = crate::core::framework::script::ScriptHostModuleDescriptor::new(
        BRIDGE_HOST_MODULE,
        BRIDGE_HOST_MODULE_VERSION,
    )
    .with_capability(BRIDGE_HOST_CAPABILITY)
    .with_documentation("Plugin bridge calls exposed to VM scripts through pre-resolved slots.");
    let mut callbacks = Vec::with_capacity(methods.len());

    for method in methods {
        let slot = bridge_table
            .resolve_slot(method.interface_id())
            .ok_or_else(|| {
                VmError::Operation(
                    RuntimeExtensionRegistryError::MissingPluginInterface(
                        method.interface_id().to_string(),
                    )
                    .to_string(),
                )
            })?;
        for capability in &method.required_capabilities {
            descriptor = descriptor.with_capability(capability.clone());
        }
        descriptor = descriptor.with_function(function_descriptor(&method));
        callbacks.push(function_callback(bridge_table.clone(), slot, method));
    }

    exports.register_module(descriptor, callbacks)?;
    Ok(())
}

pub fn register_bridge_host_module_from_manifest(
    exports: &HostExportRegistry,
    bridge_table: FrozenBridgeTable,
    manifest: &PluginPackageManifest,
    bindings: impl IntoIterator<Item = ScriptBridgeMethodBinding>,
) -> Result<(), VmError> {
    let descriptors = script_bridge_method_descriptors_from_manifest(manifest, bindings)?;
    register_bridge_host_module(exports, bridge_table, descriptors)
}

fn function_descriptor(method: &ScriptBridgeMethodDescriptor) -> ScriptHostFunctionDescriptor {
    let mut descriptor = ScriptHostFunctionDescriptor::new(
        method.function_name(),
        method.parameters.len(),
        method.parameters.len(),
        method.return_value_kind,
    );
    for parameter in &method.parameters {
        descriptor = descriptor.with_parameter(parameter.clone());
    }
    for capability in &method.required_capabilities {
        descriptor = descriptor.with_required_capability(capability.clone());
    }
    if let Some(documentation) = &method.documentation {
        descriptor = descriptor.with_documentation(documentation.clone());
    }
    descriptor
}

fn function_callback(
    bridge_table: FrozenBridgeTable,
    slot: InterfaceSlot,
    method: ScriptBridgeMethodDescriptor,
) -> HostExportFunction {
    let function_name = method.function_name.clone();
    HostExportFunction::new(function_name.clone(), move |context| {
        ensure_bridge_enabled(&bridge_table, slot, &method.interface_id)?;
        (method.method)(ScriptBridgeCall {
            interface_slot: slot,
            method_slot: method.method_slot,
            arguments: context.arguments.clone(),
        })
        .map_err(|error| {
            ScriptHostError::new(format!(
                "bridge method {}.{} failed: {}",
                BRIDGE_HOST_MODULE, function_name, error.message
            ))
        })
    })
}

fn ensure_bridge_enabled(
    bridge_table: &FrozenBridgeTable,
    slot: InterfaceSlot,
    interface_id: &str,
) -> Result<(), ScriptHostError> {
    let Some(snapshot) = bridge_table.interface_snapshot(slot) else {
        return Err(ScriptHostError::new(format!(
            "bridge interface `{interface_id}` is absent"
        )));
    };
    if snapshot.status != BridgeInterfaceStatus::Enabled {
        bridge_table.record_not_enabled_call(slot);
        return Err(ScriptHostError::new(format!(
            "bridge interface `{interface_id}` is not enabled"
        )));
    }
    bridge_table.record_enabled_call(slot);
    Ok(())
}
