use std::sync::Arc;

use zircon_runtime::core::framework::sound::{SoundError, SoundManager};

use super::model::{
    SoundEditorOutputAction, SoundEditorOutputActionReport, SoundEditorOutputDeviceRow,
    SoundEditorOutputSnapshot, SoundEditorOutputStatusModel,
};

#[derive(Clone)]
pub struct SoundEditorLiveOutputController {
    manager: Arc<dyn SoundManager>,
}

impl SoundEditorLiveOutputController {
    /// Creates a plugin-local live output controller over the neutral sound manager contract.
    pub fn new(manager: Arc<dyn SoundManager>) -> Self {
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
        ExternalAudioSourceHandle, SoundAutomationBinding, SoundAutomationBindingId,
        SoundAutomationCurve, SoundBackendCallbackBlock, SoundBackendCapability, SoundBackendState,
        SoundBackendStatus, SoundClipId, SoundClipInfo, SoundDynamicEventCatalog,
        SoundDynamicEventDelivery, SoundDynamicEventDescriptor, SoundDynamicEventExecutionReport,
        SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation, SoundEffectDescriptor,
        SoundEffectId, SoundError, SoundExternalSourceBlock, SoundHrtfProfileDescriptor,
        SoundImpulseResponseId, SoundListenerDescriptor, SoundListenerId, SoundMixBlock,
        SoundMixerGraph, SoundMixerPresetDescriptor, SoundMixerSnapshot,
        SoundOutputDeviceDescriptor, SoundOutputDeviceId, SoundOutputDeviceInfo,
        SoundOutputDeviceState, SoundOutputDeviceStatus, SoundOutputLatencyStatus,
        SoundParameterId, SoundPlaybackFinished, SoundPlaybackId, SoundPlaybackSettings,
        SoundPlaybackStatus, SoundRayTracedImpulseResponseDescriptor,
        SoundRayTracingConvolutionStatus, SoundSourceDescriptor, SoundSourceFinished,
        SoundSourceId, SoundSourceStatus, SoundTimelineSequence, SoundTimelineSequenceAdvance,
        SoundTimelineSequenceId, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
        SoundVolumeDescriptor, SoundVolumeId,
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

    impl SoundManager for FakeSoundManager {
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
            }
        }

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

        fn global_volume_gain(&self) -> Result<f32, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_global_volume_gain(&self, _gain: f32) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn default_spatial_scale(&self) -> Result<f32, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_default_spatial_scale(&self, _scale: f32) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn load_clip(&self, _locator: &str) -> Result<SoundClipId, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn clip_info(&self, _clip: SoundClipId) -> Result<SoundClipInfo, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn play_clip(
            &self,
            _clip: SoundClipId,
            _settings: SoundPlaybackSettings,
        ) -> Result<SoundPlaybackId, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn stop_playback(&self, _playback: SoundPlaybackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn pause_playback(&self, _playback: SoundPlaybackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn resume_playback(&self, _playback: SoundPlaybackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn toggle_playback(&self, _playback: SoundPlaybackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_playback_gain(
            &self,
            _playback: SoundPlaybackId,
            _gain: f32,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_playback_speed(
            &self,
            _playback: SoundPlaybackId,
            _speed: f32,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn seek_playback_seconds(
            &self,
            _playback: SoundPlaybackId,
            _seconds: f32,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn mute_playback(&self, _playback: SoundPlaybackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn unmute_playback(&self, _playback: SoundPlaybackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn toggle_mute_playback(&self, _playback: SoundPlaybackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn playback_empty(&self, _playback: SoundPlaybackId) -> Result<bool, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn playback_status(
            &self,
            _playback: SoundPlaybackId,
        ) -> Result<SoundPlaybackStatus, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn drain_finished_playbacks(&self) -> Result<Vec<SoundPlaybackFinished>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn available_mixer_presets(&self) -> Result<Vec<SoundMixerPresetDescriptor>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn apply_mixer_preset(&self, _locator: &str) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn configure_mixer(&self, _graph: SoundMixerGraph) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn mixer_snapshot(&self) -> Result<SoundMixerSnapshot, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn add_or_update_track(&self, _track: SoundTrackDescriptor) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_track(&self, _track: SoundTrackId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn add_or_update_track_send(
            &self,
            _track: SoundTrackId,
            _send: SoundTrackSend,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_track_send(
            &self,
            _track: SoundTrackId,
            _target: SoundTrackId,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn add_or_update_effect(
            &self,
            _track: SoundTrackId,
            _effect: SoundEffectDescriptor,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_effect(
            &self,
            _track: SoundTrackId,
            _effect: SoundEffectId,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn create_source(
            &self,
            _source: SoundSourceDescriptor,
        ) -> Result<SoundSourceId, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn update_source(&self, _source: SoundSourceDescriptor) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn stop_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn pause_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn resume_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn toggle_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_source_gain(&self, _source: SoundSourceId, _gain: f32) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_source_speed(&self, _source: SoundSourceId, _speed: f32) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn seek_source_seconds(
            &self,
            _source: SoundSourceId,
            _seconds: f32,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn mute_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn unmute_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn toggle_mute_source(&self, _source: SoundSourceId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn source_empty(&self, _source: SoundSourceId) -> Result<bool, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn source_status(&self, _source: SoundSourceId) -> Result<SoundSourceStatus, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn drain_finished_sources(&self) -> Result<Vec<SoundSourceFinished>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn submit_external_source_block(
            &self,
            _handle: ExternalAudioSourceHandle,
            _block: SoundExternalSourceBlock,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn clear_external_source(
            &self,
            _handle: &ExternalAudioSourceHandle,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn update_listener(&self, _listener: SoundListenerDescriptor) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_listener(&self, _listener: SoundListenerId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn update_volume(&self, _volume: SoundVolumeDescriptor) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_volume(&self, _volume: SoundVolumeId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_parameter(
            &self,
            _parameter: SoundParameterId,
            _value: f32,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn parameter_value(&self, _parameter: &SoundParameterId) -> Result<f32, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn bind_automation(&self, _binding: SoundAutomationBinding) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn apply_automation_value(
            &self,
            _binding: SoundAutomationBindingId,
            _value: f32,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn apply_automation_curve_sample(
            &self,
            _binding: SoundAutomationBindingId,
            _curve: &SoundAutomationCurve,
            _time_seconds: f32,
        ) -> Result<f32, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn unbind_automation(&self, _binding: SoundAutomationBindingId) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn schedule_timeline_sequence(
            &self,
            _sequence: SoundTimelineSequence,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_timeline_sequence(
            &self,
            _sequence: &SoundTimelineSequenceId,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn timeline_sequences(&self) -> Result<Vec<SoundTimelineSequence>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn advance_timeline_sequences(
            &self,
            _delta_seconds: f32,
        ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn dynamic_event_catalog(&self) -> Result<SoundDynamicEventCatalog, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn register_dynamic_event(
            &self,
            _descriptor: SoundDynamicEventDescriptor,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn unregister_dynamic_event(&self, _event_id: &str) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn dynamic_event_handlers(
            &self,
        ) -> Result<Vec<SoundDynamicEventHandlerDescriptor>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn register_dynamic_event_handler(
            &self,
            _handler: SoundDynamicEventHandlerDescriptor,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn unregister_dynamic_event_handler(
            &self,
            _plugin_id: &str,
            _handler_id: &str,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn submit_dynamic_event(
            &self,
            _invocation: SoundDynamicEventInvocation,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn drain_dynamic_events(&self) -> Result<Vec<SoundDynamicEventInvocation>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn dispatch_dynamic_events(&self) -> Result<Vec<SoundDynamicEventDelivery>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn execute_dynamic_events(&self) -> Result<SoundDynamicEventExecutionReport, SoundError> {
            Ok(SoundDynamicEventExecutionReport {
                executions: Vec::new(),
            })
        }

        fn set_impulse_response(
            &self,
            _impulse_response: SoundImpulseResponseId,
            _samples: Vec<f32>,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_impulse_response(
            &self,
            _impulse_response: SoundImpulseResponseId,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn load_hrtf_profile(
            &self,
            _profile: SoundHrtfProfileDescriptor,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn remove_hrtf_profile(&self, _profile_id: &str) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn hrtf_profiles(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn set_ray_tracing_convolution_status(
            &self,
            _status: SoundRayTracingConvolutionStatus,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn submit_ray_traced_impulse_response(
            &self,
            _descriptor: SoundRayTracedImpulseResponseDescriptor,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn ray_traced_impulse_responses(
            &self,
        ) -> Result<Vec<SoundRayTracedImpulseResponseDescriptor>, SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn clear_ray_traced_impulse_response(
            &self,
            _impulse_response: SoundImpulseResponseId,
        ) -> Result<(), SoundError> {
            unimplemented!("not used by live output tests")
        }

        fn render_mix(&self, _frames: usize) -> Result<SoundMixBlock, SoundError> {
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
            block_size_frames: 256,
            latency_blocks: 2,
        }
    }
}
