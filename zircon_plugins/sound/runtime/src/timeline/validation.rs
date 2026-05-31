use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundAutomationBindingId, SoundError, SoundTimelineSequence,
};

use crate::automation::curve::validate_automation_curve;
use crate::automation::values::ensure_finite_value;
use crate::engine::SoundEngineState;

pub(super) fn validate_timeline_sequence(
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
