use zircon_runtime::core::framework::sound::{SoundSourceDescriptor, SoundSourceFinishReason};

#[derive(Clone, Debug)]
pub(crate) struct SourceVoice {
    pub(crate) descriptor: SoundSourceDescriptor,
    pub(crate) cursor_frame: usize,
    pub(crate) cursor_position: f64,
    pub(crate) pending_finish: Option<SoundSourceFinishReason>,
}
