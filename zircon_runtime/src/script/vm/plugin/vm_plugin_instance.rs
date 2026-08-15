use crate::core::framework::script::ScriptHostValue;
use crate::script::{VmError, VmPluginHostContext, VmPluginManifest};

use super::super::gc_bridge::{VmGcBudget, VmGcStepOutcome};
use super::{VmStateBlob, VmStateSchema};

pub trait VmPluginInstance: Send + Sync {
    fn manifest(&self) -> &VmPluginManifest;

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        Ok(())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        Ok(VmStateBlob::default())
    }

    fn restore_state(&mut self, _state: &VmStateBlob) -> Result<(), VmError> {
        Ok(())
    }

    /// Returns the destination schema used for reflected hot-reload migration.
    fn state_schema(&mut self) -> Result<Option<VmStateSchema>, VmError> {
        Ok(None)
    }

    fn call_export(
        &mut self,
        _module_name: &str,
        _export_name: &str,
        _arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        Ok(None)
    }

    /// Performs one cooperative collection slice within the host-provided remaining budget.
    ///
    /// Implementations must retain any unfinished collector cursor in the instance and check the
    /// supplied budget at bounded work intervals. `pause_micros` is backend telemetry; the host
    /// measures wall time independently and owns the frame deadline.
    fn gc_step(&mut self, _budget: VmGcBudget) -> Result<VmGcStepOutcome, VmError> {
        Ok(VmGcStepOutcome::default())
    }
}
