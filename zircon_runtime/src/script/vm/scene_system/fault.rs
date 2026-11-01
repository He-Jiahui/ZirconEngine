//! Binding fault construction and diagnostics stay outside the typed dispatcher.
use crate::diagnostic_log::write_error;
use crate::scene::{
    bounded_script_fault_message, EntityId, LevelSystem, ScriptBindingFault, ScriptBindingFaultKind,
};
use crate::script::vm::runtime_context::ScriptHostFaultSummary;
use crate::script::{VmError, VmHostInterfaceError};

use super::{ActiveScriptBinding, ScriptSceneSystemError};

pub(super) fn export_fault(
    level: &LevelSystem,
    binding: &ActiveScriptBinding,
    export: &'static str,
    kind: ScriptBindingFaultKind,
    source: VmHostInterfaceError,
) -> ScriptBindingFault {
    let error = ScriptSceneSystemError::export_call(
        binding.binding_key.clone(),
        export,
        VmError::Operation(source.to_string()),
    );
    binding_fault(level, binding, export, kind, error.to_string(), 0)
}

pub(super) fn callback_outcome(
    level: &LevelSystem,
    binding: &ActiveScriptBinding,
    export: &'static str,
    result: Result<Option<crate::core::framework::script::ScriptHostValue>, VmHostInterfaceError>,
    summary: ScriptHostFaultSummary,
) -> Result<(), ScriptBindingFault> {
    if let Some(host) = summary.first {
        let mut message = host.message;
        if let Err(guest) = result {
            // Reserve room for secondary guest text even when the primary is already bounded.
            let mut end = message.len().min(512);
            while !message.is_char_boundary(end) {
                end -= 1;
            }
            message.truncate(end);
            message.push_str(&format!("; guest error: {guest}"));
        }
        return Err(binding_fault(
            level,
            binding,
            export,
            ScriptBindingFaultKind::HostCall {
                module: host.module,
                function: host.function,
            },
            message,
            summary.count,
        ));
    }
    result.map(|_| ()).map_err(|source| {
        export_fault(
            level,
            binding,
            export,
            ScriptBindingFaultKind::Guest,
            source,
        )
    })
}

fn binding_fault(
    level: &LevelSystem,
    binding: &ActiveScriptBinding,
    export: &str,
    kind: ScriptBindingFaultKind,
    message: String,
    host_fault_count: u64,
) -> ScriptBindingFault {
    ScriptBindingFault {
        entity: binding.entity,
        binding_key: Some(binding.binding_key.clone()),
        package: binding.package.clone(),
        module: binding.module.clone(),
        export: export.into(),
        kind,
        message: bounded_script_fault_message(message),
        host_fault_count,
        outer_frame_index: level
            .world_time()
            .last_outer_frame_index()
            .unwrap_or_default(),
    }
}

pub(super) fn record(level: &LevelSystem, fault: ScriptBindingFault) {
    if level.record_script_binding_fault(fault.clone()) {
        log(&fault);
    }
}

pub(super) fn record_invalid_row(
    level: &LevelSystem,
    entity: EntityId,
    generation: u64,
    value: &serde_json::Value,
) {
    let source =
        <serde_json::Error as serde::de::Error>::custom("expected a script.bindings array");
    let _ = record_invalid_binding(level, entity, generation, None, value, source);
}

pub(super) fn record_invalid_binding(
    level: &LevelSystem,
    entity: EntityId,
    generation: u64,
    index: Option<usize>,
    value: &serde_json::Value,
    source: serde_json::Error,
) -> Option<String> {
    let package = value
        .get("package")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let module = value
        .get("module")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let binding_key = index
        .filter(|_| {
            !package.is_empty()
                && !module.is_empty()
                && value.get("enabled").and_then(serde_json::Value::as_bool) != Some(false)
        })
        .map(|index| format!("{package}::{module}#{index}"));
    let error = ScriptSceneSystemError::invalid_binding_component(entity, source);
    let fault = ScriptBindingFault {
        entity,
        binding_key,
        package: package.into(),
        module: module.into(),
        export: String::new(),
        kind: ScriptBindingFaultKind::InvalidBinding,
        message: bounded_script_fault_message(format!("{error}; element={index:?}")),
        host_fault_count: 0,
        outer_frame_index: level
            .world_time()
            .last_outer_frame_index()
            .unwrap_or_default(),
    };
    if level.record_invalid_script_binding_fault(generation, index, fault.clone()) {
        log(&fault);
    }
    fault.binding_key
}

fn log(fault: &ScriptBindingFault) {
    // Debug formatting escapes embedded line breaks to keep exactly one diagnostic line.
    write_error("script", format!(
        "script binding fault entity={} binding={:?} export={:?} kind={:?} host_fault_count={} frame={} message={:?}",
        fault.entity, fault.binding_key, fault.export, fault.kind, fault.host_fault_count,
        fault.outer_frame_index, fault.message,
    ));
}
