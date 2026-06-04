use zircon_runtime::script::{
    VmError, VmPluginHostContext, VmPluginInstance, VmPluginManifest, VmStateBlob,
};
use zr_vm_rust_binding as zrvm;

use super::errors::{is_optional_export_missing, map_zr_error};
use super::lock::acquire_zr_vm_lock;
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
        export_name: &str,
        arguments: &[zrvm::Value],
    ) -> Result<Option<zrvm::Value>, VmError> {
        let _keep_runtime_alive = &self.runtime;
        match self
            .session
            .call_module_export(&self.entry_module, export_name, arguments)
        {
            Ok(value) => Ok(Some(value)),
            Err(error) if is_optional_export_missing(&error) => Ok(None),
            Err(error) => Err(map_zr_error(error)),
        }
    }
}

impl VmPluginInstance for ZrVmPluginInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        self.call_optional_export("activate", &[]).map(|_| ())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        self.call_optional_export("deactivate", &[]).map(|_| ())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        let _guard = acquire_zr_vm_lock();
        let value = match self.call_optional_export("saveState", &[])? {
            Some(value) => value,
            None => return Ok(VmStateBlob::default()),
        };
        match value.kind() {
            zrvm::ValueKind::String => Ok(VmStateBlob {
                bytes: value.as_string().map_err(map_zr_error)?.into_bytes(),
            }),
            zrvm::ValueKind::Null => Ok(VmStateBlob::default()),
            other => Err(VmError::Operation(format!(
                "zr_vm saveState returned unsupported value kind {other:?}"
            ))),
        }
    }

    fn restore_state(&mut self, state: &VmStateBlob) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        let state = String::from_utf8(state.bytes.clone()).map_err(|error| {
            VmError::Operation(format!("zr_vm restoreState requires UTF-8 state: {error}"))
        })?;
        let argument = zrvm::Value::new_string(&state).map_err(map_zr_error)?;
        self.call_optional_export("restoreState", &[argument])
            .map(|_| ())
    }
}
