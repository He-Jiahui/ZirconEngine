use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::core::framework::script::{
    ScriptHostCallContext, ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor,
    ScriptHostResult, ScriptHostTypeRef, ScriptHostValue,
};

use super::super::{CapabilitySet, HostHandle, VmError};
use super::HostRegistry;

pub type HostExportCallback =
    Arc<dyn Fn(&ScriptHostCallContext) -> ScriptHostResult + Send + Sync + 'static>;

#[derive(Clone)]
pub struct HostExportFunction {
    pub name: String,
    pub callback: HostExportCallback,
}

impl HostExportFunction {
    pub fn new<F>(name: impl Into<String>, callback: F) -> Self
    where
        F: Fn(&ScriptHostCallContext) -> ScriptHostResult + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            callback: Arc::new(callback),
        }
    }
}

impl fmt::Debug for HostExportFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostExportFunction")
            .field("name", &self.name)
            .field("callback", &"<host export callback>")
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostExportModuleRecord {
    pub handle: HostHandle,
    pub descriptor: ScriptHostModuleDescriptor,
}

#[derive(Clone)]
struct HostExportModuleEntry {
    record: HostExportModuleRecord,
    callbacks: HashMap<String, HostExportCallback>,
}

impl fmt::Debug for HostExportModuleEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostExportModuleEntry")
            .field("record", &self.record)
            .field("callbacks", &self.callbacks.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct HostExportRegistry {
    host_registry: HostRegistry,
    modules: Arc<Mutex<HashMap<String, HostExportModuleEntry>>>,
}

impl Default for HostExportRegistry {
    fn default() -> Self {
        Self::new(HostRegistry::default())
    }
}

impl HostExportRegistry {
    pub fn new(host_registry: HostRegistry) -> Self {
        Self {
            host_registry,
            modules: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_module(
        &self,
        descriptor: ScriptHostModuleDescriptor,
        callbacks: impl IntoIterator<Item = HostExportFunction>,
    ) -> Result<HostHandle, VmError> {
        validate_module_descriptor(&descriptor)?;
        let callbacks = collect_callbacks(&descriptor.name, callbacks)?;
        validate_callbacks(&descriptor, &callbacks)?;

        let mut modules = self.modules.lock().unwrap();
        if modules.contains_key(&descriptor.name) {
            return Err(VmError::Operation(format!(
                "host export module already registered: {}",
                descriptor.name
            )));
        }

        let handle = self
            .host_registry
            .register_capability(format!("host.module.{}", descriptor.name));
        modules.insert(
            descriptor.name.clone(),
            HostExportModuleEntry {
                record: HostExportModuleRecord { handle, descriptor },
                callbacks,
            },
        );
        Ok(handle)
    }

    pub fn module(&self, module_name: &str) -> Option<HostExportModuleRecord> {
        self.modules
            .lock()
            .unwrap()
            .get(module_name)
            .map(|entry| entry.record.clone())
    }

    pub fn modules(&self) -> Vec<HostExportModuleRecord> {
        let mut records = self
            .modules
            .lock()
            .unwrap()
            .values()
            .map(|entry| entry.record.clone())
            .collect::<Vec<_>>();
        records.sort_by(|left, right| left.descriptor.name.cmp(&right.descriptor.name));
        records
    }

    pub fn call(
        &self,
        module_name: &str,
        function_name: &str,
        arguments: Vec<ScriptHostValue>,
    ) -> Result<ScriptHostValue, VmError> {
        self.call_with_capabilities(
            module_name,
            function_name,
            arguments,
            &CapabilitySet::default(),
        )
    }

    pub fn call_with_capabilities(
        &self,
        module_name: &str,
        function_name: &str,
        arguments: Vec<ScriptHostValue>,
        granted_capabilities: &CapabilitySet,
    ) -> Result<ScriptHostValue, VmError> {
        let (descriptor, callback) = {
            let modules = self.modules.lock().unwrap();
            let entry = modules.get(module_name).ok_or_else(|| {
                VmError::Operation(format!("host export module not registered: {module_name}"))
            })?;
            let descriptor = entry
                .record
                .descriptor
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .cloned()
                .ok_or_else(|| {
                    VmError::Operation(format!(
                        "host export function not registered: {module_name}.{function_name}"
                    ))
                })?;
            let callback = entry.callbacks.get(function_name).cloned().ok_or_else(|| {
                VmError::Operation(format!(
                    "host export callback missing: {module_name}.{function_name}"
                ))
            })?;
            (descriptor, callback)
        };

        validate_call_arity(module_name, function_name, &descriptor, arguments.len())?;
        validate_call_capabilities(
            module_name,
            function_name,
            &descriptor,
            granted_capabilities,
        )?;

        let context = ScriptHostCallContext {
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            arguments,
            granted_capabilities: granted_capabilities.capabilities.clone(),
        };
        callback(&context).map_err(|error| {
            VmError::Operation(format!(
                "host export call failed: {module_name}.{function_name}: {}",
                error.message
            ))
        })
    }
}

fn collect_callbacks(
    module_name: &str,
    callbacks: impl IntoIterator<Item = HostExportFunction>,
) -> Result<HashMap<String, HostExportCallback>, VmError> {
    let mut by_name = HashMap::new();
    for callback in callbacks {
        let HostExportFunction { name, callback } = callback;
        if by_name.contains_key(&name) {
            return Err(VmError::Operation(format!(
                "duplicate host export callback: {module_name}.{name}"
            )));
        }
        by_name.insert(name, callback);
    }
    Ok(by_name)
}

fn validate_module_descriptor(descriptor: &ScriptHostModuleDescriptor) -> Result<(), VmError> {
    validate_identifier("host export module", &descriptor.name)?;
    validate_identifier("host export module version", &descriptor.version)?;
    validate_names("host export module capability", &descriptor.capabilities)?;

    let module_capabilities = descriptor
        .capabilities
        .iter()
        .cloned()
        .collect::<HashSet<_>>();

    let mut function_names = HashSet::new();
    for function in &descriptor.functions {
        validate_function_descriptor(function, &module_capabilities)?;
        if !function_names.insert(function.name.clone()) {
            return Err(VmError::Operation(format!(
                "duplicate host export function: {}.{}",
                descriptor.name, function.name
            )));
        }
    }

    let mut type_names = HashSet::new();
    for type_descriptor in &descriptor.types {
        validate_type_descriptor(&descriptor.name, type_descriptor)?;
        if !type_names.insert(type_descriptor.name.clone()) {
            return Err(VmError::Operation(format!(
                "duplicate host export type: {}.{}",
                descriptor.name, type_descriptor.name
            )));
        }
    }
    Ok(())
}

fn validate_function_descriptor(
    function: &ScriptHostFunctionDescriptor,
    module_capabilities: &HashSet<String>,
) -> Result<(), VmError> {
    validate_identifier("host export function", &function.name)?;
    if function.min_argument_count > function.max_argument_count {
        return Err(VmError::Operation(format!(
            "host export function {} has min arity greater than max arity",
            function.name
        )));
    }
    if function.parameters.len() < function.min_argument_count
        || function.parameters.len() > function.max_argument_count
    {
        return Err(VmError::Operation(format!(
            "host export function {} parameter count does not fit declared arity",
            function.name
        )));
    }
    validate_type_ref(
        "host export function return type",
        &function.return_type,
        function.return_value_kind,
    )?;
    validate_names(
        "host export function required capability",
        &function.required_capabilities,
    )?;
    for capability in &function.required_capabilities {
        if !module_capabilities.contains(capability) {
            return Err(VmError::Operation(format!(
                "host export function {} requires undeclared module capability {}",
                function.name, capability
            )));
        }
    }
    let mut parameter_names = HashSet::new();
    for parameter in &function.parameters {
        validate_identifier("host export parameter", &parameter.name)?;
        validate_type_ref(
            "host export parameter type",
            &parameter.type_ref,
            parameter.value_kind,
        )?;
        if !parameter_names.insert(parameter.name.clone()) {
            return Err(VmError::Operation(format!(
                "duplicate host export parameter: {}.{}",
                function.name, parameter.name
            )));
        }
    }
    Ok(())
}

fn validate_type_descriptor(
    module_name: &str,
    type_descriptor: &crate::core::framework::script::ScriptHostTypeDescriptor,
) -> Result<(), VmError> {
    validate_identifier("host export type", &type_descriptor.name)?;
    validate_type_ref(
        "host export type ref",
        &type_descriptor.type_ref,
        type_descriptor.value_kind,
    )?;
    if type_descriptor.type_ref.type_name != type_descriptor.name {
        return Err(VmError::Operation(format!(
            "host export type {module_name}.{} type ref {} does not match descriptor name",
            type_descriptor.name, type_descriptor.type_ref.type_name
        )));
    }
    let mut field_names = HashSet::new();
    for field in &type_descriptor.fields {
        validate_identifier("host export field", &field.name)?;
        validate_type_ref("host export field type", &field.type_ref, field.value_kind)?;
        if !field_names.insert(field.name.clone()) {
            return Err(VmError::Operation(format!(
                "duplicate host export field: {module_name}.{}.{}",
                type_descriptor.name, field.name
            )));
        }
    }
    Ok(())
}

fn validate_callbacks(
    descriptor: &ScriptHostModuleDescriptor,
    callbacks: &HashMap<String, HostExportCallback>,
) -> Result<(), VmError> {
    let function_names = descriptor
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<HashSet<_>>();

    for function in &descriptor.functions {
        if !callbacks.contains_key(&function.name) {
            return Err(VmError::Operation(format!(
                "host export callback missing for {}.{}",
                descriptor.name, function.name
            )));
        }
    }
    for callback_name in callbacks.keys() {
        if !function_names.contains(callback_name.as_str()) {
            return Err(VmError::Operation(format!(
                "host export callback provided for unknown function {}.{}",
                descriptor.name, callback_name
            )));
        }
    }
    Ok(())
}

fn validate_call_arity(
    module_name: &str,
    function_name: &str,
    descriptor: &ScriptHostFunctionDescriptor,
    argument_count: usize,
) -> Result<(), VmError> {
    if argument_count < descriptor.min_argument_count
        || argument_count > descriptor.max_argument_count
    {
        return Err(VmError::Operation(format!(
            "host export call {module_name}.{function_name} expected {}..={} arguments, received {argument_count}",
            descriptor.min_argument_count, descriptor.max_argument_count
        )));
    }
    Ok(())
}

fn validate_call_capabilities(
    module_name: &str,
    function_name: &str,
    descriptor: &ScriptHostFunctionDescriptor,
    granted_capabilities: &CapabilitySet,
) -> Result<(), VmError> {
    for capability in &descriptor.required_capabilities {
        if !granted_capabilities.capabilities.contains(capability) {
            return Err(VmError::Operation(format!(
                "host export call {module_name}.{function_name} missing capability {capability}"
            )));
        }
    }
    Ok(())
}

fn validate_names(label: &str, names: &[String]) -> Result<(), VmError> {
    let mut seen = HashSet::new();
    for name in names {
        validate_identifier(label, name)?;
        if !seen.insert(name.clone()) {
            return Err(VmError::Operation(format!("duplicate {label}: {name}")));
        }
    }
    Ok(())
}

fn validate_type_ref(
    label: &str,
    type_ref: &ScriptHostTypeRef,
    expected_kind: crate::core::framework::script::ScriptHostValueKind,
) -> Result<(), VmError> {
    validate_identifier(label, &type_ref.type_name)?;
    if type_ref.value_kind != expected_kind {
        return Err(VmError::Operation(format!(
            "{label} value kind mismatch: descriptor has {:?}, type ref has {:?}",
            expected_kind, type_ref.value_kind
        )));
    }
    Ok(())
}

fn validate_identifier(label: &str, value: &str) -> Result<(), VmError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(VmError::Operation(format!(
            "invalid {label}: `{value}` must be non-empty and already trimmed"
        )));
    }
    Ok(())
}
