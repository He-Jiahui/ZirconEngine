use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::script::{
    VmError, VmGcBudget, VmGcStepOutcome, VmPluginHostContext, VmPluginInstance, VmPluginManifest,
    VmStateBlob, VmStateSchema,
};
use zr_vm_rust_binding as zrvm;

use super::errors::{is_optional_export_missing, map_zr_error};
use super::lock::acquire_zr_vm_lock;
use super::runtime_owner::ZrVmRuntimeOwner;
use super::values::{from_zr_return_value_for_export, to_zr_value};

pub(super) fn lower_zr_arguments(
    lowered_arguments: &mut Vec<zrvm::Value>,
    arguments: &[ScriptHostValue],
) -> Result<(), zrvm::Error> {
    lowered_arguments.clear();
    lowered_arguments.reserve(arguments.len());
    for argument in arguments {
        match to_zr_value(argument) {
            Ok(value) => lowered_arguments.push(value),
            Err(error) => {
                lowered_arguments.clear();
                return Err(error);
            }
        }
    }
    Ok(())
}

pub(super) struct ZrVmPluginInstance {
    manifest: VmPluginManifest,
    runtime_owner: ZrVmRuntimeOwner,
    entry_module: String,
}

impl ZrVmPluginInstance {
    pub(super) fn new(
        manifest: VmPluginManifest,
        runtime_owner: ZrVmRuntimeOwner,
        entry_module: String,
    ) -> Self {
        Self {
            manifest,
            runtime_owner,
            entry_module,
        }
    }

    fn call_optional_export(
        runtime_owner: &mut ZrVmRuntimeOwner,
        guard: &std::sync::MutexGuard<'static, ()>,
        module_name: &str,
        export_name: &str,
        arguments: &[zrvm::Value],
    ) -> Result<Option<zrvm::Value>, VmError> {
        match runtime_owner.call_module_export(guard, module_name, export_name, arguments) {
            Ok(value) => Ok(Some(value)),
            Err(error) if is_optional_export_missing(&error) => Ok(None),
            Err(error) => Err(map_zr_error(error)),
        }
    }

    fn call_entry_lifecycle_export(
        &mut self,
        guard: &std::sync::MutexGuard<'static, ()>,
        export_name: &str,
        arguments: &[zrvm::Value],
    ) -> Result<Option<zrvm::Value>, VmError> {
        Self::call_optional_export(
            &mut self.runtime_owner,
            guard,
            &self.entry_module,
            export_name,
            arguments,
        )
    }
}

impl VmPluginInstance for ZrVmPluginInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        let guard = acquire_zr_vm_lock();
        self.call_entry_lifecycle_export(&guard, "activate", &[])
            .map(|_| ())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        let guard = acquire_zr_vm_lock();
        self.call_entry_lifecycle_export(&guard, "deactivate", &[])
            .map(|_| ())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        let guard = acquire_zr_vm_lock();
        let value = match self.call_entry_lifecycle_export(&guard, "saveState", &[])? {
            Some(value) => value,
            None => return Ok(VmStateBlob::default()),
        };
        match value.kind() {
            zrvm::ValueKind::String => {
                let snapshot = value.as_string().map_err(map_zr_error)?;
                VmStateBlob::from_json(&snapshot).map_err(Into::into)
            }
            zrvm::ValueKind::Null => Ok(VmStateBlob::default()),
            other => Err(VmError::Operation(format!(
                "zr_vm saveState returned unsupported value kind {other:?}"
            ))),
        }
    }

    fn restore_state(&mut self, state: &VmStateBlob) -> Result<(), VmError> {
        let guard = acquire_zr_vm_lock();
        let state = state.to_json()?;
        let argument = zrvm::Value::new_string(&state).map_err(map_zr_error)?;
        self.call_entry_lifecycle_export(&guard, "restoreState", &[argument])
            .map(|_| ())
    }

    fn state_schema(&mut self) -> Result<Option<VmStateSchema>, VmError> {
        let guard = acquire_zr_vm_lock();
        let value = match self.call_entry_lifecycle_export(&guard, "stateSchema", &[])? {
            Some(value) => value,
            None => return Ok(None),
        };
        match value.kind() {
            zrvm::ValueKind::String => {
                let schema = value.as_string().map_err(map_zr_error)?;
                VmStateSchema::from_json(&schema)
                    .map(Some)
                    .map_err(Into::into)
            }
            zrvm::ValueKind::Null => Ok(None),
            other => Err(VmError::Operation(format!(
                "zr_vm stateSchema returned unsupported value kind {other:?}"
            ))),
        }
    }

    fn gc_step(&mut self, budget: VmGcBudget) -> Result<VmGcStepOutcome, VmError> {
        let guard = acquire_zr_vm_lock();
        let outcome = self
            .runtime_owner
            .gc_step(&guard, budget.max_micros_per_frame)
            .map_err(map_zr_error)?;
        Ok(VmGcStepOutcome {
            pause_micros: outcome.pause_micros,
            root_count: outcome.root_count,
            cross_boundary_reference_count: outcome.cross_boundary_reference_count,
        })
    }

    fn call_export(
        &mut self,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        let guard = acquire_zr_vm_lock();
        let mut lowered_arguments = self.runtime_owner.take_lowered_arguments(&guard);
        if let Err(error) = lower_zr_arguments(&mut lowered_arguments, arguments) {
            self.runtime_owner
                .recycle_lowered_arguments(&guard, lowered_arguments);
            return Err(map_zr_error(error));
        }
        let call_result = Self::call_optional_export(
            &mut self.runtime_owner,
            &guard,
            module_name,
            export_name,
            &lowered_arguments,
        );
        self.runtime_owner
            .recycle_lowered_arguments(&guard, lowered_arguments);
        let Some(value) = call_result? else {
            return Ok(None);
        };
        from_zr_return_value_for_export(&value, module_name, export_name)
            .map(Some)
            .map_err(map_zr_error)
    }
}
