use zircon_runtime::core::framework::sound::{
    SoundError, SoundTimelineSequence, SoundTimelineSequenceAdvance, SoundTimelineSequenceId,
};

use crate::timeline::advance::advance_timeline_sequences;
use crate::timeline::schedule::{remove_timeline_sequence, schedule_timeline_sequence};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn schedule_timeline_sequence_impl(
        &self,
        sequence: SoundTimelineSequence,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        schedule_timeline_sequence(&mut state, sequence)
    }

    pub(super) fn remove_timeline_sequence_impl(
        &self,
        sequence: &SoundTimelineSequenceId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        remove_timeline_sequence(&mut state, sequence)
    }

    pub(super) fn timeline_sequences_impl(&self) -> Result<Vec<SoundTimelineSequence>, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .timeline_sequences
            .iter()
            .map(|playback| playback.sequence.clone())
            .collect())
    }

    pub(super) fn advance_timeline_sequences_impl(
        &self,
        delta_seconds: f32,
    ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        advance_timeline_sequences(&mut state, delta_seconds)
    }
}
