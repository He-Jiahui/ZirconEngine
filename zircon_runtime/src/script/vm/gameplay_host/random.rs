//! Gameplay draws borrow the Runtime's seeded stream for the active World/entity generation.
use crate::core::framework::random::{
    RandomEntityKey, RandomPurposeKey, RandomStreamKey, RandomSystemKey, RandomWorldKey,
};
use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostError, ScriptHostValue, ScriptHostValueRef,
};
use crate::script::runtime_context_for_frame;

// Stable namespace tags ("gameplay" and "range_f8"); changing them changes replay sequences.
const GAMEPLAY_RANDOM_SYSTEM: RandomSystemKey = RandomSystemKey::new(0x6761_6d65_706c_6179);
const RANDOM_RANGE_PURPOSE: RandomPurposeKey = RandomPurposeKey::new(0x7261_6e67_655f_6638);
const UNIT_F64_DENOMINATOR: f64 = 9_007_199_254_740_992.0;

pub(super) fn random_range(
    frame: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let min = range_bound(frame, 0)?;
    let max = range_bound(frame, 1)?;
    if !min.is_finite() || !max.is_finite() || min > max {
        return Err(ScriptHostError::new(
            "random_range requires finite bounds with min <= max",
        ));
    }

    let runtime = runtime_context_for_frame(frame)?;
    let core = runtime.core_handle()?;
    let key = runtime.level.with_world_and_replacement_epoch(
        |world, replacement_epoch| -> Result<_, ScriptHostError> {
            let entity = world.internal_entity(runtime.entity).ok_or_else(|| {
                ScriptHostError::new(format!("random_range entity {} is missing", runtime.entity))
            })?;
            // Generations are slot-local; include the slot so a stable ID reinserted elsewhere
            // cannot resume the old incarnation's stream.
            let incarnation = (u64::from(entity.index()) << 32) | u64::from(entity.generation());
            Ok(RandomStreamKey::for_entity(
                RandomWorldKey::new(runtime.level.handle().get(), replacement_epoch),
                RandomEntityKey::new(runtime.entity, incarnation),
                GAMEPLAY_RANDOM_SYSTEM,
                RANDOM_RANGE_PURPOSE,
                0,
            ))
        },
    )?;
    if min == max {
        return Ok(ScriptHostValue::Float(min));
    }

    let mut stream = core
        .random_service()
        .acquire_stream(key)
        .map_err(|error| ScriptHostError::new(format!("random_range: {error}")))?;
    // The f64 sample uses two engine draws. Refuse an incomplete sample before advancing.
    if stream.draw_index().checked_add(2).is_none() {
        return Err(ScriptHostError::new(
            "random_range random stream draw index is exhausted",
        ));
    }
    let high = stream
        .try_next_u32()
        .map_err(|error| ScriptHostError::new(format!("random_range: {error}")))?;
    let low = stream
        .try_next_u32()
        .map_err(|error| ScriptHostError::new(format!("random_range: {error}")))?;
    let fraction =
        (((u64::from(high)) << 21) | (u64::from(low) >> 11)) as f64 / UNIT_F64_DENOMINATOR;

    let width = max - min;
    let sample = if width.is_finite() {
        min + width * fraction
    } else {
        // Opposite finite extremes can have an infinite difference; their weighted sum cannot.
        min * (1.0 - fraction) + max * fraction
    };
    // Floating-point rounding can reach max for a narrow range. Preserve [min, max).
    Ok(ScriptHostValue::Float(sample.clamp(min, max.next_down())))
}

fn range_bound(frame: &ScriptHostCallFrame<'_>, index: usize) -> Result<f64, ScriptHostError> {
    frame.arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::Float(value) => Ok(value),
        ScriptHostValueRef::Int(value) => Ok(value as f64),
        value => Err(ScriptHostError::new(format!(
            "random_range argument {index} expected float, received {:?}",
            value.kind()
        ))),
    })
}
