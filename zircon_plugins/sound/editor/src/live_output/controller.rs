//! 实时输出控制器把中立声音服务的设备状态投影成编辑器模型，操作后尽力重新取快照供界面反馈。
use std::sync::Arc;

use zircon_runtime::core::framework::sound::{
    SoundBackendManager, SoundError, SoundOutputDeviceManager,
};

use super::model::{
    SoundEditorOutputAction, SoundEditorOutputActionReport, SoundEditorOutputDeviceRow,
    SoundEditorOutputSnapshot, SoundEditorOutputStatusModel,
};

#[derive(Clone)]
pub struct SoundEditorLiveOutputController {
    manager: Arc<dyn SoundEditorLiveOutputManager>,
}

impl SoundEditorLiveOutputController {
    /// Creates a plugin-local live output controller over the neutral output control contracts.
    pub fn new(manager: Arc<dyn SoundEditorLiveOutputManager>) -> Self {
        Self { manager }
    }

    /// Projects the current output picker rows, backend state, and device status for editor UI.
    pub fn snapshot(&self) -> Result<SoundEditorOutputSnapshot, SoundError> {
        let backend = self.manager.backend_status();
        let status = self.manager.output_device_status()?;
        let mut diagnostics = Vec::new();
        let devices = match self.manager.available_output_devices() {
            Ok(devices) => devices,
            Err(error) => {
                diagnostics.push(format!("failed to enumerate sound output devices: {error}"));
                Vec::new()
            }
        };
        let selected = status.descriptor.clone();
        let status = SoundEditorOutputStatusModel::from_status(status, &backend);
        diagnostics.extend(status.diagnostics.iter().cloned());
        if let Some(detail) = backend.detail.clone() {
            diagnostics.push(detail);
        }
        dedupe_diagnostics(&mut diagnostics);

        Ok(SoundEditorOutputSnapshot {
            devices: devices
                .into_iter()
                .map(|device| SoundEditorOutputDeviceRow::from_info(device, &selected))
                .collect(),
            status,
            backend,
            diagnostics,
        })
    }

    /// Applies one output action and returns a refreshed best-effort snapshot for the editor.
    pub fn apply_action(&self, action: SoundEditorOutputAction) -> SoundEditorOutputActionReport {
        let result = match &action {
            SoundEditorOutputAction::Refresh => Ok(()),
            SoundEditorOutputAction::Configure(descriptor) => {
                self.manager.configure_output_device(descriptor.clone())
            }
            SoundEditorOutputAction::Start => self.manager.start_output_device(),
            SoundEditorOutputAction::Stop => self.manager.stop_output_device(),
        };

        match result {
            Ok(()) => match self.snapshot() {
                Ok(snapshot) => SoundEditorOutputActionReport::success(action, snapshot),
                Err(error) => {
                    SoundEditorOutputActionReport::failure(action, error.to_string(), None)
                }
            },
            Err(error) => {
                let mut snapshot = self.snapshot().ok();
                if let Some(snapshot) = snapshot.as_mut() {
                    push_diagnostic(&mut snapshot.diagnostics, error.to_string());
                }
                SoundEditorOutputActionReport::failure(action, error.to_string(), snapshot)
            }
        }
    }
}

/// 编辑器只依赖中立后端状态与设备控制接口；实现者不需要向界面暴露 Kira 句柄。
pub trait SoundEditorLiveOutputManager:
    SoundBackendManager + SoundOutputDeviceManager + Send + Sync
{
}

impl<T> SoundEditorLiveOutputManager for T where
    T: SoundBackendManager + SoundOutputDeviceManager + Send + Sync
{
}

fn dedupe_diagnostics(diagnostics: &mut Vec<String>) {
    let mut unique = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics.drain(..) {
        if !unique.iter().any(|entry| entry == &diagnostic) {
            unique.push(diagnostic);
        }
    }
    *diagnostics = unique;
}

fn push_diagnostic(diagnostics: &mut Vec<String>, diagnostic: String) {
    if !diagnostics.iter().any(|entry| entry == &diagnostic) {
        diagnostics.push(diagnostic);
    }
}

#[cfg(test)]
#[path = "tests/controller.rs"]
mod tests;
