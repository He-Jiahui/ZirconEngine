use super::{
    host_registry::HostRegistry, vm_backend::VmBackend, vm_error::VmError,
    vm_plugin_instance::VmPluginInstance, vm_plugin_manifest::VmPluginManifest,
    vm_plugin_package::VmPluginPackage, vm_state_blob::VmStateBlob,
};

#[derive(Debug, Default)]
pub struct MockVmBackend;

impl VmBackend for MockVmBackend {
    fn backend_name(&self) -> &str {
        "mock"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: HostRegistry,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(MockVmPluginInstance {
            manifest: package.manifest.clone(),
            state: VmStateBlob::default(),
            activations: 0,
        }))
    }
}

#[derive(Debug)]
struct MockVmPluginInstance {
    manifest: VmPluginManifest,
    state: VmStateBlob,
    activations: usize,
}

impl VmPluginInstance for MockVmPluginInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &HostRegistry) -> Result<(), VmError> {
        self.activations += 1;
        Ok(())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        Ok(self.state.clone())
    }

    fn restore_state(&mut self, state: &VmStateBlob) -> Result<(), VmError> {
        self.state = state.clone();
        Ok(())
    }
}
