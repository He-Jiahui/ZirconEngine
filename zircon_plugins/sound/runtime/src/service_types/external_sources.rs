//! 外部音源按句柄保存最新的完整块，提交会替换旧块；当前 Kira 播放入口仍拒绝外部输入，保存成功不表示已进入设备输出。
use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundError, SoundExternalSourceBlock,
};

use crate::descriptor_validation::external_source::{
    validate_external_source_block, validate_external_source_handle,
};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn submit_external_source_block_impl(
        &self,
        handle: ExternalAudioSourceHandle,
        block: SoundExternalSourceBlock,
    ) -> Result<(), SoundError> {
        validate_external_source_handle(&handle)?;
        validate_external_source_block(&block)?;
        crate::poison_recovery::lock_recover(&self.state)
            .external_sources
            .insert(handle, block);
        Ok(())
    }

    pub(super) fn clear_external_source_impl(
        &self,
        handle: &ExternalAudioSourceHandle,
    ) -> Result<(), SoundError> {
        validate_external_source_handle(handle)?;
        crate::poison_recovery::lock_recover(&self.state)
            .external_sources
            .remove(handle)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownExternalSource {
                handle: handle.clone(),
            })
    }
}
