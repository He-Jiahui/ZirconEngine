use zircon_runtime::core::framework::sound::{
    SoundBackendCallbackBlock, SoundBackendCapability, SoundBackendState, SoundBackendStatus,
    SoundError, SoundMixBlock, SoundOutputDeviceDescriptor, SoundOutputDeviceInfo,
    SoundOutputDeviceStatus,
};

use super::DefaultSoundManager;
use crate::output::{available_output_backends, available_output_devices};

impl DefaultSoundManager {
    pub(super) fn backend_name_impl(&self) -> String {
        let config = self.config();
        if !config.enabled {
            return "disabled".to_string();
        }
        let unavailable_backend = {
            let state = self.state.lock().expect("sound state mutex poisoned");
            state
                .output_device
                .unavailable_backend_status()
                .map(|(backend, _)| backend.to_string())
        };
        unavailable_backend.unwrap_or(config.backend)
    }

    pub(super) fn backend_status_impl(&self) -> SoundBackendStatus {
        let config = self.config();
        if !config.enabled {
            return SoundBackendStatus {
                requested_backend: config.backend,
                active_backend: None,
                state: SoundBackendState::Disabled,
                detail: Some("sound playback is disabled".to_string()),
                sample_rate_hz: config.sample_rate_hz,
                channel_count: config.channel_count,
            };
        }

        let unavailable_backend = {
            let state = self.state.lock().expect("sound state mutex poisoned");
            state
                .output_device
                .unavailable_backend_status()
                .map(|(backend, detail)| (backend.to_string(), detail.to_string()))
        };
        if let Some((backend, detail)) = unavailable_backend {
            return SoundBackendStatus {
                requested_backend: backend,
                active_backend: None,
                state: SoundBackendState::Unavailable,
                detail: Some(detail),
                sample_rate_hz: config.sample_rate_hz,
                channel_count: config.channel_count,
            };
        }

        SoundBackendStatus {
            requested_backend: config.backend.clone(),
            active_backend: Some(config.backend),
            state: SoundBackendState::Ready,
            detail: None,
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count,
        }
    }

    pub(super) fn configure_output_device_impl(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        let mut config = self.config.lock().expect("sound config mutex poisoned");
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if let Err(error) = state.output_device.configure(descriptor.clone()) {
            if let SoundError::BackendUnavailable { detail } = &error {
                state
                    .output_device
                    .record_backend_unavailable(descriptor.backend, detail.clone());
            }
            return Err(error);
        }
        config.backend = descriptor.backend.clone();
        config.sample_rate_hz = descriptor.sample_rate_hz;
        config.channel_count = descriptor.channel_count;
        config.block_size_frames = descriptor.block_size_frames;
        state.graph.sample_rate_hz = config.sample_rate_hz;
        state.graph.channel_count = config.channel_count;
        state.effect_states.clear();
        state.track_states.clear();
        state.hrtf_states.clear();
        Ok(())
    }

    pub(super) fn start_output_device_impl(&self) -> Result<(), SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .start_with_engine(self.state.clone(), self.config.clone())?;
        Ok(())
    }

    pub(super) fn stop_output_device_impl(&self) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .stop();
        Ok(())
    }

    pub(super) fn output_device_status_impl(&self) -> Result<SoundOutputDeviceStatus, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .status())
    }

    pub(super) fn available_output_devices_impl(
        &self,
    ) -> Result<Vec<SoundOutputDeviceInfo>, SoundError> {
        Ok(available_output_devices(&self.config()))
    }

    pub(super) fn render_output_device_block_impl(&self) -> Result<SoundMixBlock, SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }

        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let frames = state.output_device.block_size_frames()?;
        match state.render_mix(&config, frames) {
            Ok(block) => {
                let sample_count = block.samples.len();
                state
                    .output_device
                    .record_rendered_block(frames, sample_count);
                Ok(block)
            }
            Err(error) => {
                state.output_device.record_error(&error);
                Err(error)
            }
        }
    }

    pub(super) fn available_output_backends_impl(
        &self,
    ) -> Result<Vec<SoundBackendCapability>, SoundError> {
        Ok(available_output_backends())
    }

    pub(super) fn pull_output_backend_callback_impl(
        &self,
    ) -> Result<SoundBackendCallbackBlock, SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }

        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let frames = state.output_device.block_size_frames()?;
        match state.render_mix(&config, frames) {
            Ok(block) => {
                let sample_count = block.samples.len();
                let channel_count = block.channel_count as usize;
                let rendered_frames = if channel_count == 0 {
                    0
                } else {
                    sample_count / channel_count
                };
                let report = state.output_device.record_callback_block(
                    frames,
                    rendered_frames,
                    sample_count,
                );
                Ok(SoundBackendCallbackBlock { report, block })
            }
            Err(error) => {
                state.output_device.record_callback_error(frames, &error);
                Err(error)
            }
        }
    }
}
