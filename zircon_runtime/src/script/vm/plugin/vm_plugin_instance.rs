use crate::core::framework::script::ScriptHostValue;
use crate::script::{VmError, VmPluginHostContext, VmPluginManifest, VmStateBlob};

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

    fn call_export(
        &mut self,
        _module_name: &str,
        _export_name: &str,
        _arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        Ok(None)
    }
}
