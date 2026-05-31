use zircon_runtime::core::framework::sound::{
    SoundAutomationCurve, SoundAutomationInterpolation, SoundAutomationKeyframe, SoundError,
};

use super::values::ensure_finite_value;

pub(crate) fn sample_automation_curve(
    curve: &SoundAutomationCurve,
    time_seconds: f32,
) -> Result<f32, SoundError> {
    ensure_finite_value("automation curve time", time_seconds)?;
    validate_automation_curve(curve)?;

    let first = curve
        .keyframes
        .first()
        .expect("validated automation curve has at least one keyframe");
    if time_seconds <= first.time_seconds {
        return Ok(first.value);
    }

    for window in curve.keyframes.windows(2) {
        let start = window[0];
        let end = window[1];
        if time_seconds <= end.time_seconds {
            return Ok(interpolate_automation_value(start, end, time_seconds));
        }
    }

    Ok(curve
        .keyframes
        .last()
        .expect("validated automation curve has at least one keyframe")
        .value)
}

pub(crate) fn validate_automation_curve(curve: &SoundAutomationCurve) -> Result<(), SoundError> {
    if curve.keyframes.is_empty() {
        return Err(SoundError::InvalidParameter(
            "automation curve requires at least one keyframe".to_string(),
        ));
    }

    let mut previous_time = None;
    for keyframe in &curve.keyframes {
        ensure_finite_value("automation keyframe time", keyframe.time_seconds)?;
        ensure_finite_value("automation keyframe value", keyframe.value)?;
        if let Some(previous_time) = previous_time {
            if keyframe.time_seconds <= previous_time {
                return Err(SoundError::InvalidParameter(
                    "automation curve keyframes must be strictly increasing".to_string(),
                ));
            }
        }
        previous_time = Some(keyframe.time_seconds);
    }
    Ok(())
}

fn interpolate_automation_value(
    start: SoundAutomationKeyframe,
    end: SoundAutomationKeyframe,
    time_seconds: f32,
) -> f32 {
    match start.interpolation {
        SoundAutomationInterpolation::Step => start.value,
        SoundAutomationInterpolation::Linear | SoundAutomationInterpolation::SmoothStep => {
            let span = (end.time_seconds - start.time_seconds).max(f32::EPSILON);
            let mut amount = ((time_seconds - start.time_seconds) / span).clamp(0.0, 1.0);
            if start.interpolation == SoundAutomationInterpolation::SmoothStep {
                amount = amount * amount * (3.0 - 2.0 * amount);
            }
            start.value + (end.value - start.value) * amount
        }
    }
}
