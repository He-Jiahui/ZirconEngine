use std::sync::Arc;

use crate::core::framework::bridge::{BridgeInterfaceStatus, BridgeInvocationTable, InterfaceSlot};
use crate::core::framework::script::{
    ScriptHostArguments, ScriptHostError, ScriptHostFunctionDescriptor,
    ScriptHostParameterDescriptor, ScriptHostResult, ScriptHostValueKind,
};

use super::super::VmError;
use super::{HostExportFunction, HostExportRegistry};

pub const BRIDGE_HOST_MODULE: &str = "zr.zircon.bridge";
pub const BRIDGE_HOST_CAPABILITY: &str = "bridge.call";
const BRIDGE_HOST_MODULE_VERSION: &str = "0.1.0";

pub type ScriptBridgeMethodFn =
    Arc<dyn for<'call> Fn(ScriptBridgeCall<'call>) -> ScriptHostResult + Send + Sync + 'static>;

pub struct ScriptBridgeCall<'call> {
    pub interface_slot: InterfaceSlot,
    pub method_slot: u32,
    pub arguments: &'call ScriptHostArguments<'call>,
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
        F: for<'call> Fn(ScriptBridgeCall<'call>) -> ScriptHostResult + Send + Sync + 'static,
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
        insert_required_capability(&mut self.required_capabilities, capability.into());
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
}

fn insert_required_capability(required_capabilities: &mut Vec<String>, capability: String) {
    if let Err(index) = required_capabilities.binary_search(&capability) {
        required_capabilities.insert(index, capability);
    }
}

pub fn register_bridge_host_module<Table>(
    exports: &HostExportRegistry,
    bridge_table: Table,
    methods: impl IntoIterator<Item = ScriptBridgeMethodDescriptor>,
) -> Result<(), VmError>
where
    Table: BridgeInvocationTable,
{
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
            .resolve_interface_slot(method.interface_id())
            .ok_or_else(|| {
                VmError::Operation(format!(
                    "missing plugin interface `{}`",
                    method.interface_id()
                ))
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

fn function_callback<Table>(
    bridge_table: Table,
    slot: InterfaceSlot,
    method: ScriptBridgeMethodDescriptor,
) -> HostExportFunction
where
    Table: BridgeInvocationTable,
{
    let function_name = method.function_name.clone();
    HostExportFunction::new(function_name.clone(), move |context| {
        ensure_bridge_enabled(&bridge_table, slot, &method.interface_id)?;
        (method.method)(ScriptBridgeCall {
            interface_slot: slot,
            method_slot: method.method_slot,
            arguments: &context.arguments,
        })
        .map_err(|error| {
            ScriptHostError::new(format!(
                "bridge method {}.{} failed: {}",
                BRIDGE_HOST_MODULE, function_name, error.message
            ))
        })
    })
}

fn ensure_bridge_enabled<Table>(
    bridge_table: &Table,
    slot: InterfaceSlot,
    interface_id: &str,
) -> Result<(), ScriptHostError>
where
    Table: BridgeInvocationTable,
{
    let status = bridge_table.interface_status_at(slot);
    if status == BridgeInterfaceStatus::Absent {
        return Err(ScriptHostError::new(format!(
            "bridge interface `{interface_id}` is absent"
        )));
    }
    if status != BridgeInterfaceStatus::Enabled {
        bridge_table.record_not_enabled_call(slot);
        return Err(ScriptHostError::new(format!(
            "bridge interface `{interface_id}` is not enabled"
        )));
    }
    bridge_table.record_enabled_call(slot);
    Ok(())
}

#[cfg(test)]
#[path = "bridge_host_module/capability_insert_tests.rs"]
mod capability_insert_tests;
