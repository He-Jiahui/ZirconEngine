use super::super::{
    SoundClipId, SoundClipInfo, SoundError, SoundPlaybackFinished, SoundPlaybackId,
    SoundPlaybackSettings, SoundPlaybackStatus,
};

pub trait SoundPlaybackManager {
    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError>;
    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError>;
    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError>;
    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn pause_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn resume_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn toggle_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn set_playback_gain(&self, playback: SoundPlaybackId, gain: f32) -> Result<(), SoundError>;
    fn set_playback_speed(&self, playback: SoundPlaybackId, speed: f32) -> Result<(), SoundError>;
    fn seek_playback_seconds(
        &self,
        playback: SoundPlaybackId,
        seconds: f32,
    ) -> Result<(), SoundError>;
    fn mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn unmute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn toggle_mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn playback_empty(&self, playback: SoundPlaybackId) -> Result<bool, SoundError>;
    fn playback_status(&self, playback: SoundPlaybackId)
        -> Result<SoundPlaybackStatus, SoundError>;
    fn drain_finished_playbacks(&self) -> Result<Vec<SoundPlaybackFinished>, SoundError>;
}
