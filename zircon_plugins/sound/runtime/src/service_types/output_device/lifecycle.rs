use zircon_runtime::core::framework::sound::SoundError;

use super::super::DefaultSoundManager;
use crate::output::{validate_backend_supported, validate_output_device_descriptor};
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn start_output_device_impl(&self) -> Result<(), SoundError> {
        let config = lock_recover(&self.config);
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }
        let mut state = lock_recover(&self.state);
        let descriptor = state.output_device.descriptor().clone();
        let graph = state.graph.clone();
        validate_output_device_descriptor(&descriptor)?;
        validate_backend_supported(&descriptor)?;
        if state.kira.is_active() {
            state.output_device.mark_started();
            return Ok(());
        }
        if let Err(error) = state.kira.activate_output(&descriptor, &config) {
            record_start_failure(&mut state, &descriptor.backend, &error);
            return Err(error);
        }
        if let Err(error) = state.kira.sync_graph(&graph) {
            record_start_failure(&mut state, &descriptor.backend, &error);
            return Err(error);
        }
        if let Err(error) = super::super::sources::sync_preconfigured_sources(&mut state) {
            record_start_failure(&mut state, &descriptor.backend, &error);
            return Err(error);
        }
        state.output_device.mark_started();
        Ok(())
    }

    pub(in crate::service_types) fn stop_output_device_impl(&self) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.deactivate_kira();
        state.output_device.stop();
        Ok(())
    }
}

fn record_start_failure(
    state: &mut crate::engine::SoundEngineState,
    backend: &str,
    error: &SoundError,
) {
    state.deactivate_kira();
    state
        .output_device
        .record_backend_unavailable(backend.to_string(), error.to_string());
}
