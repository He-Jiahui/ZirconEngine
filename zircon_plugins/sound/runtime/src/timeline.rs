use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundAutomationBindingId, SoundError, SoundTimelineAutomationSample, SoundTimelineSequence,
    SoundTimelineSequenceAdvance, SoundTimelineSequenceId,
};

use crate::automation::{
    apply_automation_target, ensure_finite_value, sample_automation_curve,
    validate_automation_curve,
};
use crate::engine::SoundEngineState;

#[derive(Clone, Debug)]
pub(crate) struct SoundTimelineSequencePlayback {
    pub(crate) sequence: SoundTimelineSequence,
    pub(crate) time_seconds: f32,
}

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

fn validate_timeline_sequence(
    state: &SoundEngineState,
    sequence: &SoundTimelineSequence,
) -> Result<(), SoundError> {
    if sequence.id.as_str().trim().is_empty() {
        return Err(SoundError::InvalidParameter(
            "timeline sequence requires a non-empty id".to_string(),
        ));
    }
    ensure_finite_value("timeline sequence duration", sequence.duration_seconds)?;
    if sequence.duration_seconds <= 0.0 {
        return Err(SoundError::InvalidParameter(
            "timeline sequence duration must be positive".to_string(),
        ));
    }
    if sequence.tracks.is_empty() {
        return Err(SoundError::InvalidParameter(
            "timeline sequence requires at least one automation track".to_string(),
        ));
    }

    let mut bindings = HashSet::<SoundAutomationBindingId>::new();
    for track in &sequence.tracks {
        if !bindings.insert(track.binding) {
            return Err(SoundError::InvalidParameter(
                "timeline sequence contains duplicate automation bindings".to_string(),
            ));
        }
        if !state.automation_bindings.contains_key(&track.binding) {
            return Err(SoundError::UnknownAutomationBinding {
                binding: track.binding,
            });
        }
        validate_automation_curve(&track.curve)?;
        if track.curve.keyframes.iter().any(|keyframe| {
            keyframe.time_seconds < 0.0 || keyframe.time_seconds > sequence.duration_seconds
        }) {
            return Err(SoundError::InvalidParameter(
                "timeline sequence keyframes must fall within the sequence duration".to_string(),
            ));
        }
    }
    Ok(())
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
