use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor,
    ScriptHostOwnedArgumentSource, ScriptHostResult, ScriptHostTypeRef, ScriptHostValue,
};

use super::super::{CapabilitySet, HostHandle, VmError};
use super::script_call_table::{ScriptCallTable, ScriptCallTableBuilder};
use super::HostRegistry;

pub type HostExportCallback = Arc<
    dyn for<'frame> Fn(&ScriptHostCallFrame<'frame>) -> ScriptHostResult + Send + Sync + 'static,
>;

#[derive(Clone)]
pub struct HostExportFunction {
    pub name: String,
    pub callback: HostExportCallback,
}

impl HostExportFunction {
    pub fn new<F>(name: impl Into<String>, callback: F) -> Self
    where
        F: for<'frame> Fn(&ScriptHostCallFrame<'frame>) -> ScriptHostResult + Send + Sync + 'static,
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
    state: Arc<Mutex<HostExportRegistryState>>,
}

#[derive(Debug, Default)]
struct HostExportRegistryState {
    generation: u64,
    modules: HashMap<String, HostExportModuleEntry>,
    call_table: ScriptCallTable,
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
            state: Arc::new(Mutex::new(HostExportRegistryState::default())),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, HostExportRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn register_module(
        &self,
        descriptor: ScriptHostModuleDescriptor,
        callbacks: impl IntoIterator<Item = HostExportFunction>,
    ) -> Result<HostHandle, VmError> {
        validate_module_descriptor(&descriptor)?;
        let callbacks = collect_callbacks(&descriptor.name, callbacks)?;
        validate_callbacks(&descriptor, &callbacks)?;

        let mut state = self.lock_state();
        if state.modules.contains_key(&descriptor.name) {
            return Err(VmError::Operation(format!(
                "host export module already registered: {}",
                descriptor.name
            )));
        }
        let next_generation = state.generation.checked_add(1).ok_or_else(|| {
            VmError::Operation("host export registry generation exhausted".to_string())
        })?;

        let handle = self
            .host_registry
            .register_capability(format!("host.module.{}", descriptor.name))
            .map_err(|error| VmError::Operation(error.to_string()))?;
        state.modules.insert(
            descriptor.name.clone(),
            HostExportModuleEntry {
                record: HostExportModuleRecord { handle, descriptor },
                callbacks,
            },
        );
        let call_table = build_script_call_table(next_generation, &state.modules);
        state.generation = next_generation;
        state.call_table = call_table;
        Ok(handle)
    }

    pub fn module(&self, module_name: &str) -> Option<HostExportModuleRecord> {
        self.lock_state()
            .modules
            .get(module_name)
            .map(|entry| entry.record.clone())
    }

    pub fn modules(&self) -> Vec<HostExportModuleRecord> {
        let mut records = self
            .lock_state()
            .modules
            .values()
            .map(|entry| entry.record.clone())
            .collect::<Vec<_>>();
        records.sort_by(|left, right| left.descriptor.name.cmp(&right.descriptor.name));
        records
    }

    pub fn script_call_table(&self) -> ScriptCallTable {
        self.lock_state().call_table.clone()
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
        let call_table = {
            let state = self.lock_state();
            if !state.modules.contains_key(module_name) {
                return Err(VmError::Operation(format!(
                    "host export module not registered: {module_name}"
                )));
            }
            state.call_table.clone()
        };
        let call_site = call_table
            .resolve(module_name, function_name)
            .ok_or_else(|| {
                VmError::Operation(format!(
                    "host export function not registered: {module_name}.{function_name}"
                ))
            })?;
        let source = ScriptHostOwnedArgumentSource::new(&arguments);
        call_site.call(
            crate::core::framework::script::ScriptHostArguments::new(&source),
            granted_capabilities,
        )
    }
}

fn build_script_call_table(
    generation: u64,
    modules: &HashMap<String, HostExportModuleEntry>,
) -> ScriptCallTable {
    let mut entries = modules.values().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.record
            .descriptor
            .name
            .cmp(&right.record.descriptor.name)
    });

    let mut builder = ScriptCallTableBuilder::new(generation);
    for entry in entries {
        let module_name = Arc::<str>::from(entry.record.descriptor.name.as_str());
        for function in &entry.record.descriptor.functions {
            let callback = entry
                .callbacks
                .get(&function.name)
                .expect("validated host export callback must remain registered")
                .clone();
            builder.add(module_name.clone(), function.clone(), callback);
        }
    }
    builder.build()
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

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::core::framework::script::{ScriptHostFunctionDescriptor, ScriptHostValueKind};

    use super::*;

    #[test]
    fn host_export_registry_accessors_recover_poisoned_module_lock() {
        let registry = HostExportRegistry::default();

        let poison_result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = registry.state.lock().unwrap();
            panic!("poison host export registry");
        }));
        assert!(poison_result.is_err());

        let descriptor = ScriptHostModuleDescriptor::new("test.host", "1").with_function(
            ScriptHostFunctionDescriptor::new("ping", 0, 0, ScriptHostValueKind::Null),
        );
        let handle = registry
            .register_module(
                descriptor,
                [HostExportFunction::new("ping", |_| {
                    Ok(ScriptHostValue::Null)
                })],
            )
            .unwrap();

        assert!(registry.module("test.host").is_some());
        assert_eq!(registry.modules()[0].handle, handle);
        assert_eq!(
            registry.call("test.host", "ping", Vec::new()).unwrap(),
            ScriptHostValue::Null
        );
        let call_table = registry.script_call_table();
        let call_site = call_table.resolve("test.host", "ping").unwrap();
        assert_eq!(
            call_site
                .call(Vec::new(), &CapabilitySet::default())
                .unwrap(),
            ScriptHostValue::Null
        );
    }
}
