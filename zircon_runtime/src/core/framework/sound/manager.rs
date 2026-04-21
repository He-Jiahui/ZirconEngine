use super::{
    SoundBackendStatus, SoundClipId, SoundClipInfo, SoundError, SoundMixBlock, SoundPlaybackId,
    SoundPlaybackSettings,
};

pub trait SoundManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn backend_status(&self) -> SoundBackendStatus;
    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError>;
    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError>;
    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError>;
    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError>;
}
