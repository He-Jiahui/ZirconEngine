use crate::{HostRegistry, VmError, VmPluginManifest, VmStateBlob};

pub trait VmPluginInstance: Send + Sync {
    fn manifest(&self) -> &VmPluginManifest;

    fn activate(&mut self, _host: &HostRegistry) -> Result<(), VmError> {
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
}
