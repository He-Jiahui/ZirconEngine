//! 推进器按调度顺序采样并应用自动化目标，成功时生成每条序列的完成报告及后续时间。
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

    // BUG: [CR-SOUND-AUDIT-0002] 取出全部序列后，采样或目标应用若经 ? 提前返回，尚未回写的调度表保持空；调度后解绑即可触发 UnknownAutomationBinding。证据：automation_timeline.rs。
    let mut scheduled = std::mem::take(&mut state.timeline_sequences);
    let scheduled_count = scheduled.len();
    let mut retained = Vec::with_capacity(scheduled_count);
    let mut reports = Vec::with_capacity(scheduled_count);
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
    let track_count = sequence.tracks.len();
    let mut samples = Vec::with_capacity(track_count);
    let mut applications = Vec::with_capacity(track_count);
    for track in &sequence.tracks {
        let value = sample_automation_curve(&track.curve, time_seconds)?;
        let binding = state.automation_bindings.get(&track.binding).ok_or(
            SoundError::UnknownAutomationBinding {
                binding: track.binding,
            },
        )?;
        samples.push(SoundTimelineAutomationSample {
            binding: track.binding,
            value,
        });
        applications.push((binding.target.clone(), binding.parameter.clone(), value));
    }
    for (target, parameter, value) in applications {
        apply_automation_target(state, target, &parameter, value)?;
    }
    Ok(samples)
}

#[cfg(test)]
#[path = "tests/advance.rs"]
mod tests;
