use zircon_runtime::core::framework::sound::{SoundOutputDeviceDescriptor, SoundOutputDeviceState};

use crate::SoundConfig;

#[derive(Debug)]
pub(crate) struct SoundOutputDeviceRuntimeState {
    pub(super) descriptor: SoundOutputDeviceDescriptor,
    pub(super) state: SoundOutputDeviceState,
    pub(super) rendered_blocks: u64,
    pub(super) rendered_frames: u64,
    pub(super) callback_count: u64,
    pub(super) last_callback_sequence: Option<u64>,
    pub(super) next_callback_sequence: u64,
    pub(super) underrun_count: u64,
    pub(super) last_error: Option<String>,
    pub(super) unavailable_backend: Option<String>,
    pub(super) unavailable_detail: Option<String>,
}

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn descriptor(&self) -> &SoundOutputDeviceDescriptor {
        &self.descriptor
    }

    pub(crate) fn unavailable_backend_status(&self) -> Option<(&str, &str)> {
        self.unavailable_backend
            .as_deref()
            .zip(self.unavailable_detail.as_deref())
    }

    pub(crate) fn record_backend_unavailable(&mut self, backend: String, detail: String) {
        self.state = SoundOutputDeviceState::Stopped;
        self.last_error = Some(detail.clone());
        self.unavailable_backend = Some(backend);
        self.unavailable_detail = Some(detail);
    }

    pub(crate) fn new(config: &SoundConfig) -> Self {
        let descriptor = SoundOutputDeviceDescriptor {
            id: zircon_runtime::core::framework::sound::SoundOutputDeviceId::default_system(),
            backend: config.backend.clone(),
            display_name: "Default Output".to_string(),
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count.max(1),
            channel_layout: config.channel_layout.clone(),
            block_size_frames: config.block_size_frames,
            latency_blocks: 2,
        };

        Self {
            descriptor,
            state: SoundOutputDeviceState::Stopped,
            rendered_blocks: 0,
            rendered_frames: 0,
            callback_count: 0,
            last_callback_sequence: None,
            next_callback_sequence: 0,
            underrun_count: 0,
            last_error: None,
            unavailable_backend: None,
            unavailable_detail: None,
        }
    }
}
