use zircon_runtime::core::framework::sound::{
    SoundError, SoundTimelineAutomationSample, SoundTimelineSequence, SoundTimelineSequenceAdvance,
};

use crate::automation::curve::sample_automation_curve;
use crate::automation::target::apply_automation_target;
use crate::automation::values::ensure_finite_value;
use crate::engine::SoundEngineState;

pub(crate) fn advance_timeline_sequences(
    state: &mut SoundEngineState,
    delta_seconds: f32,
) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
    ensure_finite_value("timeline sequence delta", delta_seconds)?;
    if delta_seconds < 0.0 {
        return Err(SoundError::InvalidParameter(
            "timeline sequence delta must be non-negative".to_string(),
        ));
    }

    let mut scheduled = std::mem::take(&mut state.timeline_sequences);
    let mut retained = Vec::new();
    let mut reports = Vec::new();
    for mut playback in scheduled.drain(..) {
        let raw_time = playback.time_seconds + delta_seconds;
        let (sample_time, completed) = resolve_sample_time(
            playback.sequence.duration_seconds,
            raw_time,
            playback.sequence.looping,
        );
        let samples = apply_timeline_sequence_at(state, &playback.sequence, sample_time)?;
        reports.push(SoundTimelineSequenceAdvance {
            sequence: playback.sequence.id.clone(),
            time_seconds: sample_time,
            completed,
            samples,
        });
        if !completed {
            playback.time_seconds = sample_time;
            retained.push(playback);
        }
    }
    state.timeline_sequences = retained;
    Ok(reports)
}

fn resolve_sample_time(duration_seconds: f32, time_seconds: f32, looping: bool) -> (f32, bool) {
    if looping {
        (time_seconds.rem_euclid(duration_seconds), false)
    } else {
        (
            time_seconds.min(duration_seconds),
            time_seconds >= duration_seconds,
        )
    }
}

fn apply_timeline_sequence_at(
    state: &mut SoundEngineState,
    sequence: &SoundTimelineSequence,
    time_seconds: f32,
) -> Result<Vec<SoundTimelineAutomationSample>, SoundError> {
    let mut samples = Vec::new();
    let mut applications = Vec::new();
    for track in &sequence.tracks {
        let value = sample_automation_curve(&track.curve, time_seconds)?;
        let binding = state
            .automation_bindings
            .get(&track.binding)
            .cloned()
            .ok_or(SoundError::UnknownAutomationBinding {
                binding: track.binding,
            })?;
        samples.push(SoundTimelineAutomationSample {
            binding: track.binding,
            value,
        });
        applications.push((binding.target, binding.parameter, value));
    }
    for (target, parameter, value) in applications {
        apply_automation_target(state, target, &parameter, value)?;
    }
    Ok(samples)
}
