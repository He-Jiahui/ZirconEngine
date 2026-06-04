use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundClipInfo, SoundError, SoundPlaybackFinished, SoundPlaybackId,
    SoundPlaybackManager, SoundPlaybackSettings, SoundPlaybackStatus,
};

use super::super::DefaultSoundManager;

impl SoundPlaybackManager for DefaultSoundManager {
    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError> {
        self.load_clip_impl(locator)
    }

    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError> {
        self.clip_info_impl(clip)
    }

    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError> {
        self.play_clip_impl(clip, settings)
    }

    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.stop_playback_impl(playback)
    }

    fn pause_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.pause_playback_impl(playback)
    }

    fn resume_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.resume_playback_impl(playback)
    }

    fn toggle_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.toggle_playback_impl(playback)
    }

    fn set_playback_gain(&self, playback: SoundPlaybackId, gain: f32) -> Result<(), SoundError> {
        self.set_playback_gain_impl(playback, gain)
    }

    fn set_playback_speed(&self, playback: SoundPlaybackId, speed: f32) -> Result<(), SoundError> {
        self.set_playback_speed_impl(playback, speed)
    }

    fn seek_playback_seconds(
        &self,
        playback: SoundPlaybackId,
        seconds: f32,
    ) -> Result<(), SoundError> {
        self.seek_playback_seconds_impl(playback, seconds)
    }

    fn mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.mute_playback_impl(playback)
    }

    fn unmute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.unmute_playback_impl(playback)
    }

    fn toggle_mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.toggle_mute_playback_impl(playback)
    }

    fn playback_empty(&self, playback: SoundPlaybackId) -> Result<bool, SoundError> {
        self.playback_empty_impl(playback)
    }

    fn playback_status(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<SoundPlaybackStatus, SoundError> {
        self.playback_status_impl(playback)
    }

    fn drain_finished_playbacks(&self) -> Result<Vec<SoundPlaybackFinished>, SoundError> {
        self.drain_finished_playbacks_impl()
    }
}
