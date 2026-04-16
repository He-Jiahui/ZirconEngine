use super::{
    host_registry::HostRegistry, vm_error::VmError, vm_plugin_manifest::VmPluginManifest,
    vm_state_blob::VmStateBlob,
};

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
