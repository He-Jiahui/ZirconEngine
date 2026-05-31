use zircon_runtime::asset::SoundAsset;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundPlaybackCompletionAction, SoundTrackId,
};

#[derive(Clone, Debug)]
pub(crate) struct LoadedClip {
    pub(crate) asset: SoundAsset,
}

#[derive(Clone, Debug)]
pub(crate) struct ActivePlayback {
    pub(crate) clip: SoundClipId,
    pub(crate) cursor_frame: usize,
    pub(crate) cursor_position: f64,
    pub(crate) gain: f32,
    pub(crate) speed: f32,
    pub(crate) looped: bool,
    pub(crate) completion_action: SoundPlaybackCompletionAction,
    pub(crate) paused: bool,
    pub(crate) muted: bool,
    pub(crate) range_start_frame: usize,
    pub(crate) range_end_frame: Option<usize>,
    pub(crate) output_track: SoundTrackId,
    pub(crate) pan: f32,
}
