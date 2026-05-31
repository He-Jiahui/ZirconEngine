use zircon_runtime::core::framework::sound::{
    SoundBackendCallbackBlock, SoundError, SoundMixBlock,
};

use super::DefaultSoundManager;

impl DefaultSoundManager {
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
