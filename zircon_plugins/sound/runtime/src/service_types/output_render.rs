//! 框架保留手动取块接口供能力探测；当前 Kira 独占回调，调用者必须处理明确的未支持结果。
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
