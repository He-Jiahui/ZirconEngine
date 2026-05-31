use zircon_runtime::core::framework::sound::{
    SoundError, SoundTimelineSequence, SoundTimelineSequenceId,
};

use crate::engine::SoundEngineState;

use super::playback::SoundTimelineSequencePlayback;
use super::validation::validate_timeline_sequence;

pub(crate) fn schedule_timeline_sequence(
    state: &mut SoundEngineState,
    sequence: SoundTimelineSequence,
) -> Result<(), SoundError> {
    validate_timeline_sequence(state, &sequence)?;
    if let Some(existing) = state
        .timeline_sequences
        .iter_mut()
        .find(|playback| playback.sequence.id == sequence.id)
    {
        *existing = SoundTimelineSequencePlayback {
            sequence,
            time_seconds: 0.0,
        };
    } else {
        state
            .timeline_sequences
            .push(SoundTimelineSequencePlayback {
                sequence,
                time_seconds: 0.0,
            });
    }
    Ok(())
}

pub(crate) fn remove_timeline_sequence(
    state: &mut SoundEngineState,
    sequence: &SoundTimelineSequenceId,
) -> Result<(), SoundError> {
    let before = state.timeline_sequences.len();
    state
        .timeline_sequences
        .retain(|playback| &playback.sequence.id != sequence);
    if before == state.timeline_sequences.len() {
        return Err(SoundError::UnknownTimelineSequence {
            sequence: sequence.clone(),
        });
    }
    Ok(())
}
