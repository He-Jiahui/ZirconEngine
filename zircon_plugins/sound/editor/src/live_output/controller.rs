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
mod tests {
    use std::sync::{Arc, Mutex};

    use zircon_runtime::core::framework::sound::{
        AudioChannelLayout, SoundBackendCallbackBlock, SoundBackendCapability, SoundBackendState,
        SoundBackendStatus, SoundMixBlock, SoundOutputDeviceDescriptor, SoundOutputDeviceId,
        SoundOutputDeviceInfo, SoundOutputDeviceState, SoundOutputDeviceStatus,
        SoundOutputLatencyStatus,
    };

    use super::*;

    #[test]
    fn live_output_snapshot_marks_selected_device_and_projects_status() {
        let manager = Arc::new(FakeSoundManager::default());
        let controller = SoundEditorLiveOutputController::new(manager.clone());

        let snapshot = controller.snapshot().unwrap();

        assert_eq!(snapshot.devices.len(), 2);
        assert!(snapshot.devices.iter().any(|device| {
            device.selected && device.descriptor.id.as_str() == "sound.output.software"
        }));
        assert_eq!(snapshot.status.state, SoundOutputDeviceState::Stopped);
        assert_eq!(snapshot.status.backend_state, SoundBackendState::Ready);
        assert_eq!(snapshot.status.latency.estimated_latency_frames, 512);
        assert_eq!(snapshot.status.last_callback_sequence, Some(3));
        assert!(snapshot
            .diagnostics
            .iter()
            .any(|entry| entry == "status ok"));
    }

    #[test]
    fn live_output_actions_configure_start_stop_and_refresh() {
        let manager = Arc::new(FakeSoundManager::default());
        let controller = SoundEditorLiveOutputController::new(manager.clone());
        let cpal = cpal_descriptor();

        let configure = controller.apply_action(SoundEditorOutputAction::Configure(cpal.clone()));
        assert!(configure.success, "{:?}", configure.error);
        assert!(configure.snapshot.unwrap().devices.iter().any(|device| {
            device.selected && device.descriptor.id.as_str() == cpal.id.as_str()
        }));

        let start = controller.apply_action(SoundEditorOutputAction::Start);
        assert!(start.success, "{:?}", start.error);
        assert_eq!(
            start.snapshot.unwrap().status.state,
            SoundOutputDeviceState::Started
        );

        let stop = controller.apply_action(SoundEditorOutputAction::Stop);
        assert!(stop.success, "{:?}", stop.error);
        assert_eq!(
            stop.snapshot.unwrap().status.state,
            SoundOutputDeviceState::Stopped
        );
        assert_eq!(
            manager.calls.lock().unwrap().as_slice(),
            &["configure", "start", "stop"]
        );
    }

    #[test]
    fn live_output_action_failure_returns_best_effort_snapshot() {
        let manager = Arc::new(FakeSoundManager::default());
        manager
            .fail_start
            .lock()
            .unwrap()
            .replace("cpal device missing".to_string());
        let controller = SoundEditorLiveOutputController::new(manager);

        let report = controller.apply_action(SoundEditorOutputAction::Start);

        assert!(!report.success);
        assert_eq!(
            report.error.as_deref(),
            Some("sound backend unavailable: cpal device missing")
        );
        let snapshot = report
            .snapshot
            .expect("failure should keep best-effort state");
        assert!(snapshot
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("cpal device missing")));
    }

    #[derive(Debug)]
    struct FakeSoundManager {
        descriptor: Mutex<SoundOutputDeviceDescriptor>,
        state: Mutex<SoundOutputDeviceState>,
        calls: Mutex<Vec<&'static str>>,
        fail_start: Mutex<Option<String>>,
    }

    impl Default for FakeSoundManager {
        fn default() -> Self {
            Self {
                descriptor: Mutex::new(software_descriptor()),
                state: Mutex::new(SoundOutputDeviceState::Stopped),
                calls: Mutex::new(Vec::new()),
                fail_start: Mutex::new(None),
            }
        }
    }

    impl SoundBackendManager for FakeSoundManager {
        fn backend_name(&self) -> String {
            self.descriptor.lock().unwrap().backend.clone()
        }

        fn backend_status(&self) -> SoundBackendStatus {
            let descriptor = self.descriptor.lock().unwrap();
            SoundBackendStatus {
                requested_backend: descriptor.backend.clone(),
                active_backend: Some(descriptor.backend.clone()),
                state: SoundBackendState::Ready,
                detail: None,
                sample_rate_hz: descriptor.sample_rate_hz,
                channel_count: descriptor.channel_count,
                channel_layout: descriptor.channel_layout.clone(),
            }
        }
    }

    impl SoundOutputDeviceManager for FakeSoundManager {
        fn configure_output_device(
            &self,
            descriptor: SoundOutputDeviceDescriptor,
        ) -> Result<(), SoundError> {
            self.calls.lock().unwrap().push("configure");
            *self.descriptor.lock().unwrap() = descriptor;
            *self.state.lock().unwrap() = SoundOutputDeviceState::Stopped;
            Ok(())
        }

        fn start_output_device(&self) -> Result<(), SoundError> {
            self.calls.lock().unwrap().push("start");
            if let Some(detail) = self.fail_start.lock().unwrap().clone() {
                return Err(SoundError::BackendUnavailable { detail });
            }
            *self.state.lock().unwrap() = SoundOutputDeviceState::Started;
            Ok(())
        }

        fn stop_output_device(&self) -> Result<(), SoundError> {
            self.calls.lock().unwrap().push("stop");
            *self.state.lock().unwrap() = SoundOutputDeviceState::Stopped;
            Ok(())
        }

        fn output_device_status(&self) -> Result<SoundOutputDeviceStatus, SoundError> {
            let descriptor = self.descriptor.lock().unwrap().clone();
            Ok(SoundOutputDeviceStatus {
                descriptor,
                state: *self.state.lock().unwrap(),
                latency: SoundOutputLatencyStatus {
                    requested_latency_blocks: 2,
                    estimated_latency_frames: 512,
                    estimated_latency_seconds: 512.0 / 48_000.0,
                    queued_samples: Some(128),
                    capacity_samples: Some(1024),
                },
                rendered_blocks: 3,
                rendered_frames: 768,
                callback_count: 4,
                last_callback_sequence: Some(3),
                underrun_count: 0,
                last_error: None,
                diagnostics: vec!["status ok".to_string()],
            })
        }

        fn available_output_devices(&self) -> Result<Vec<SoundOutputDeviceInfo>, SoundError> {
            Ok(vec![
                SoundOutputDeviceInfo {
                    descriptor: software_descriptor(),
                    is_default: true,
                    available: true,
                    diagnostic: None,
                },
                SoundOutputDeviceInfo {
                    descriptor: cpal_descriptor(),
                    is_default: false,
                    available: true,
                    diagnostic: Some("host default".to_string()),
                },
            ])
        }

        fn render_output_device_block(&self) -> Result<SoundMixBlock, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn available_output_backends(&self) -> Result<Vec<SoundBackendCapability>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn pull_output_backend_callback(&self) -> Result<SoundBackendCallbackBlock, SoundError> {
            unimplemented!("not used by live output tests")
        }
    }

    fn software_descriptor() -> SoundOutputDeviceDescriptor {
        SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.software"),
            backend: "software-null".to_string(),
            display_name: "Software Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: AudioChannelLayout::stereo(),
            block_size_frames: 256,
            latency_blocks: 2,
        }
    }

    fn cpal_descriptor() -> SoundOutputDeviceDescriptor {
        SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.default"),
            backend: "cpal".to_string(),
            display_name: "CPAL Default Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: AudioChannelLayout::stereo(),
            block_size_frames: 256,
            latency_blocks: 2,
        }
    }
}
