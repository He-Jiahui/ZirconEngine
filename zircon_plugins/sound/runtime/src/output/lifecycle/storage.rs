use zircon_runtime::core::framework::sound::{SoundOutputDeviceDescriptor, SoundOutputDeviceState};

use crate::SoundConfig;

use super::session::SoundOutputBackendSession;

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
    pub(super) backend_session: SoundOutputBackendSession,
}

impl SoundOutputDeviceRuntimeState {
    pub(crate) fn new(config: &SoundConfig) -> Self {
        Self {
            descriptor: SoundOutputDeviceDescriptor::software(
                config.backend.clone(),
                config.sample_rate_hz,
                config.channel_count,
                config.block_size_frames,
            ),
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
            backend_session: SoundOutputBackendSession::None,
        }
    }
}
