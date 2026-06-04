use super::super::{
    ExternalAudioSourceHandle, SoundError, SoundExternalSourceBlock, SoundListenerDescriptor,
    SoundListenerId, SoundSourceDescriptor, SoundSourceFinished, SoundSourceId, SoundSourceStatus,
    SoundVolumeDescriptor, SoundVolumeId,
};

pub trait SoundSourceManager {
    fn create_source(&self, source: SoundSourceDescriptor) -> Result<SoundSourceId, SoundError>;
    fn update_source(&self, source: SoundSourceDescriptor) -> Result<(), SoundError>;
    fn remove_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn stop_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn pause_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn resume_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn toggle_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn set_source_gain(&self, source: SoundSourceId, gain: f32) -> Result<(), SoundError>;
    fn set_source_speed(&self, source: SoundSourceId, speed: f32) -> Result<(), SoundError>;
    fn seek_source_seconds(&self, source: SoundSourceId, seconds: f32) -> Result<(), SoundError>;
    fn mute_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn unmute_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn toggle_mute_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn source_empty(&self, source: SoundSourceId) -> Result<bool, SoundError>;
    fn source_status(&self, source: SoundSourceId) -> Result<SoundSourceStatus, SoundError>;
    fn drain_finished_sources(&self) -> Result<Vec<SoundSourceFinished>, SoundError>;
    fn submit_external_source_block(
        &self,
        handle: ExternalAudioSourceHandle,
        block: SoundExternalSourceBlock,
    ) -> Result<(), SoundError>;
    fn clear_external_source(&self, handle: &ExternalAudioSourceHandle) -> Result<(), SoundError>;
    fn update_listener(&self, listener: SoundListenerDescriptor) -> Result<(), SoundError>;
    fn remove_listener(&self, listener: SoundListenerId) -> Result<(), SoundError>;
    fn update_volume(&self, volume: SoundVolumeDescriptor) -> Result<(), SoundError>;
    fn remove_volume(&self, volume: SoundVolumeId) -> Result<(), SoundError>;
}
