use zircon_runtime::core::framework::sound::{
    SoundBackendCallbackBlock, SoundError, SoundMixBlock,
};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn render_output_device_block_impl(&self) -> Result<SoundMixBlock, SoundError> {
        Err(SoundError::UnsupportedAdvancedFeature(
            "manual mix rendering was retired; Kira owns the output callback".to_string(),
        ))
    }

    pub(super) fn pull_output_backend_callback_impl(
        &self,
    ) -> Result<SoundBackendCallbackBlock, SoundError> {
        Err(SoundError::UnsupportedAdvancedFeature(
            "manual backend callbacks were retired; Kira owns the output callback".to_string(),
        ))
    }
}
