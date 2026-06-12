use crate::core::framework::script::ScriptHostValue;
use crate::script::{
    VmBackend, VmError, VmPluginHostContext, VmPluginInstance, VmPluginManifest, VmPluginPackage,
    VmStateBlob,
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
        _host: &VmPluginHostContext,
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

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
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

    fn call_export(
        &mut self,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        let event = serde_json::json!({
            "module": module_name,
            "export": export_name,
            "arguments": arguments,
        });
        let mut calls = mock_call_log_from_state(&self.state)?;
        calls.push(event);
        self.state.bytes = serde_json::to_vec(&calls).map_err(|error| {
            VmError::Operation(format!("mock vm call log encode failed: {error}"))
        })?;
        Ok(Some(ScriptHostValue::Null))
    }
}

fn mock_call_log_from_state(state: &VmStateBlob) -> Result<Vec<serde_json::Value>, VmError> {
    if state.bytes.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&state.bytes)
        .map_err(|error| VmError::Operation(format!("mock vm call log decode failed: {error}")))
}
