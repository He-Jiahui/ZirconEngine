use crate::core::framework::script::ScriptHostValue;
use crate::diagnostic_log::write_log;
use crate::script::{
    VmError, VmPluginHostContext, VmPluginInstance, VmPluginManifest, VmStateBlob, VmStateSchema,
};
use zr_vm_rust_binding as zrvm;

use super::errors::{is_optional_export_missing, map_zr_error};
use super::lock::acquire_zr_vm_lock;
use super::values::{from_zr_return_value_for_export, to_zr_value};
use super::ZrVmRegistration;

pub(super) struct ZrVmPluginInstance {
    manifest: VmPluginManifest,
    session: zrvm::ProjectSession,
    _registrations: Vec<ZrVmRegistration>,
    runtime: zrvm::Runtime,
    entry_module: String,
}

unsafe impl Send for ZrVmPluginInstance {}
unsafe impl Sync for ZrVmPluginInstance {}

impl ZrVmPluginInstance {
    pub(super) fn new(
        manifest: VmPluginManifest,
        session: zrvm::ProjectSession,
        registrations: Vec<ZrVmRegistration>,
        runtime: zrvm::Runtime,
        entry_module: String,
    ) -> Self {
        Self {
            manifest,
            session,
            _registrations: registrations,
            runtime,
            entry_module,
        }
    }

    fn call_optional_export(
        &mut self,
        module_name: &str,
        export_name: &str,
        arguments: &[zrvm::Value],
    ) -> Result<Option<zrvm::Value>, VmError> {
        let _keep_runtime_alive = &self.runtime;
        match self
            .session
            .call_module_export(module_name, export_name, arguments)
        {
            Ok(value) => Ok(Some(value)),
            Err(error) if is_optional_export_missing(&error) => Ok(None),
            Err(error) => Err(map_zr_error(error)),
        }
    }

    fn call_entry_lifecycle_export(
        &mut self,
        export_name: &str,
        arguments: &[zrvm::Value],
    ) -> Result<Option<zrvm::Value>, VmError> {
        let entry_module = self.entry_module.clone();
        self.call_optional_export(&entry_module, export_name, arguments)
    }
}

impl VmPluginInstance for ZrVmPluginInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        write_log(
            "zr_vm_project_backend",
            format!(
                "zr_vm_project_activate_start package={}",
                self.manifest.name
            ),
        );
        let result = self
            .call_entry_lifecycle_export("activate", &[])
            .map(|_| ());
        write_log(
            "zr_vm_project_backend",
            format!(
                "zr_vm_project_activate_done package={} success={}",
                self.manifest.name,
                result.is_ok()
            ),
        );
        result
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        self.call_entry_lifecycle_export("deactivate", &[])
            .map(|_| ())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        let _guard = acquire_zr_vm_lock();
        let value = match self.call_entry_lifecycle_export("saveState", &[])? {
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
        let _guard = acquire_zr_vm_lock();
        let state = state.to_json()?;
        let argument = zrvm::Value::new_string(&state).map_err(map_zr_error)?;
        self.call_entry_lifecycle_export("restoreState", &[argument])
            .map(|_| ())
    }

    fn state_schema(&mut self) -> Result<Option<VmStateSchema>, VmError> {
        let _guard = acquire_zr_vm_lock();
        let value = match self.call_entry_lifecycle_export("stateSchema", &[])? {
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

    fn call_export(
        &mut self,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        let _guard = acquire_zr_vm_lock();
        let export_label = format!("{module_name}.{export_name}");
        write_log(
            "zr_vm_project_backend",
            format!(
                "zr_vm_project_export_start package={} export={export_label}",
                self.manifest.name
            ),
        );
        let arguments = arguments
            .iter()
            .cloned()
            .map(to_zr_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_zr_error)?;
        let Some(value) = self.call_optional_export(module_name, export_name, &arguments)? else {
            write_log(
                "zr_vm_project_backend",
                format!(
                    "zr_vm_project_export_missing package={} export={export_label}",
                    self.manifest.name
                ),
            );
            return Ok(None);
        };
        let result = from_zr_return_value_for_export(&value, &export_label)
            .map(Some)
            .map_err(map_zr_error);
        write_log(
            "zr_vm_project_backend",
            format!(
                "zr_vm_project_export_done package={} export={export_label} success={}",
                self.manifest.name,
                result.is_ok()
            ),
        );
        result
    }
}
