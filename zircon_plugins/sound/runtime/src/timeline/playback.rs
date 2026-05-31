use zircon_runtime::core::framework::sound::SoundTimelineSequence;

#[derive(Clone, Debug)]
pub(crate) struct SoundTimelineSequencePlayback {
    pub(crate) sequence: SoundTimelineSequence,
    pub(crate) time_seconds: f32,
}
