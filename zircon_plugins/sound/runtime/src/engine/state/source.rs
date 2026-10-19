//! 声源描述是可保存的意图，Kira 播放句柄只在输出活动时绑定，停机后仍保留描述供重建。
use zircon_runtime::core::framework::sound::{
    SoundPlaybackId, SoundSourceDescriptor, SoundSourceFinishReason,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceVoice {
    pub(crate) descriptor: SoundSourceDescriptor,
    // Control-thread spatial projection, independent of the saved user gain.
    pub(crate) environment_gain: f32,
    pub(crate) environment_pan: f32,
    pub(crate) cursor_frame: usize,
    pub(crate) cursor_position: f64,
    pub(crate) pending_finish: Option<SoundSourceFinishReason>,
    pub(crate) kira_playback: Option<SoundPlaybackId>,
}

impl SourceVoice {
    pub(crate) fn new(descriptor: SoundSourceDescriptor) -> Self {
        Self {
            descriptor,
            environment_gain: 1.0,
            environment_pan: 0.0,
            cursor_frame: 0,
            cursor_position: 0.0,
            pending_finish: None,
            kira_playback: None,
        }
    }
}
