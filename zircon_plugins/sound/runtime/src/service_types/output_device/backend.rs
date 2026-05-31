use zircon_runtime::core::framework::sound::{SoundBackendState, SoundBackendStatus};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn backend_name_impl(&self) -> String {
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

    pub(in crate::service_types) fn backend_status_impl(&self) -> SoundBackendStatus {
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
}
