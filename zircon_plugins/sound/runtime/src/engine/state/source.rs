use zircon_runtime::core::framework::sound::{
    SoundPlaybackId, SoundSourceDescriptor, SoundSourceFinishReason,
};

#[derive(Clone, Debug)]
pub(crate) struct SourceVoice {
    pub(crate) descriptor: SoundSourceDescriptor,
    pub(crate) cursor_frame: usize,
    pub(crate) cursor_position: f64,
    pub(crate) pending_finish: Option<SoundSourceFinishReason>,
    pub(crate) kira_playback: Option<SoundPlaybackId>,
}

impl SourceVoice {
    pub(crate) fn new(descriptor: SoundSourceDescriptor) -> Self {
        Self {
            descriptor,
            cursor_frame: 0,
            cursor_position: 0.0,
            pending_finish: None,
            kira_playback: None,
        }
    }
}
