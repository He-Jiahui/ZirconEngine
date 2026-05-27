use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundError, SoundExternalSourceBlock,
};

use crate::descriptor_validation::{
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
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .external_sources
            .insert(handle, block);
        Ok(())
    }

    pub(super) fn clear_external_source_impl(
        &self,
        handle: &ExternalAudioSourceHandle,
    ) -> Result<(), SoundError> {
        validate_external_source_handle(handle)?;
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .external_sources
            .remove(handle)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownExternalSource {
                handle: handle.clone(),
            })
    }
}
