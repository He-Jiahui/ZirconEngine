use super::SoundDelayLineState;

#[derive(Clone, Debug, Default)]
pub(crate) struct SoundTrackRuntimeState {
    pub(crate) control_delay_line: SoundDelayLineState,
}
