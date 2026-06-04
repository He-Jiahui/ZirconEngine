use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundError, SoundExternalSourceBlock, SoundListenerDescriptor,
    SoundListenerId, SoundSourceDescriptor, SoundSourceFinished, SoundSourceId, SoundSourceManager,
    SoundSourceStatus, SoundVolumeDescriptor, SoundVolumeId,
};

use super::super::DefaultSoundManager;

impl SoundSourceManager for DefaultSoundManager {
    fn create_source(&self, source: SoundSourceDescriptor) -> Result<SoundSourceId, SoundError> {
        self.create_source_impl(source)
    }

    fn update_source(&self, source: SoundSourceDescriptor) -> Result<(), SoundError> {
        self.update_source_impl(source)
    }

    fn remove_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.remove_source_impl(source)
    }

    fn stop_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.stop_source_impl(source)
    }

    fn pause_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.pause_source_impl(source)
    }

    fn resume_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.resume_source_impl(source)
    }

    fn toggle_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.toggle_source_impl(source)
    }

    fn set_source_gain(&self, source: SoundSourceId, gain: f32) -> Result<(), SoundError> {
        self.set_source_gain_impl(source, gain)
    }

    fn set_source_speed(&self, source: SoundSourceId, speed: f32) -> Result<(), SoundError> {
        self.set_source_speed_impl(source, speed)
    }

    fn seek_source_seconds(&self, source: SoundSourceId, seconds: f32) -> Result<(), SoundError> {
        self.seek_source_seconds_impl(source, seconds)
    }

    fn mute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.mute_source_impl(source)
    }

    fn unmute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.unmute_source_impl(source)
    }

    fn toggle_mute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.toggle_mute_source_impl(source)
    }

    fn source_empty(&self, source: SoundSourceId) -> Result<bool, SoundError> {
        self.source_empty_impl(source)
    }

    fn source_status(&self, source: SoundSourceId) -> Result<SoundSourceStatus, SoundError> {
        self.source_status_impl(source)
    }

    fn drain_finished_sources(&self) -> Result<Vec<SoundSourceFinished>, SoundError> {
        self.drain_finished_sources_impl()
    }

    fn submit_external_source_block(
        &self,
        handle: ExternalAudioSourceHandle,
        block: SoundExternalSourceBlock,
    ) -> Result<(), SoundError> {
        self.submit_external_source_block_impl(handle, block)
    }

    fn clear_external_source(&self, handle: &ExternalAudioSourceHandle) -> Result<(), SoundError> {
        self.clear_external_source_impl(handle)
    }

    fn update_listener(&self, listener: SoundListenerDescriptor) -> Result<(), SoundError> {
        self.update_listener_impl(listener)
    }

    fn remove_listener(&self, listener: SoundListenerId) -> Result<(), SoundError> {
        self.remove_listener_impl(listener)
    }

    fn update_volume(&self, volume: SoundVolumeDescriptor) -> Result<(), SoundError> {
        self.update_volume_impl(volume)
    }

    fn remove_volume(&self, volume: SoundVolumeId) -> Result<(), SoundError> {
        self.remove_volume_impl(volume)
    }
}
