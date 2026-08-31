use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationParameterValue, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};

use super::super::AnimationDocumentMutationError;

const TRANSITION_FRAMES_PER_SECOND: f32 = 30.0;

pub(super) fn create_state(
    asset: &mut AnimationStateMachineAsset,
    state_name: &str,
    graph_locator: &str,
) -> Result<bool, AnimationDocumentMutationError> {
    if asset.states.iter().any(|state| state.name == state_name) {
        return Ok(false);
    }
    let graph = AssetReference::from_locator(AssetUri::parse(graph_locator).map_err(|error| {
        AnimationDocumentMutationError::InvalidGraphLocator {
            message: error.to_string(),
        }
    })?);
    asset
        .states
        .push(AnimationStateAsset::graph_ref(state_name, graph));
    Ok(true)
}

pub(super) fn remove_state(asset: &mut AnimationStateMachineAsset, state_name: &str) -> bool {
    let before = asset.states.len();
    asset.states.retain(|state| state.name != state_name);
    if before == asset.states.len() {
        return false;
    }
    asset.transitions.retain(|transition| {
        transition.from_state != state_name && transition.to_state != state_name
    });
    if asset.entry_state == state_name {
        asset.entry_state = asset
            .states
            .first()
            .map(|state| state.name.clone())
            .unwrap_or_default();
    }
    true
}

pub(super) fn set_entry_state(asset: &mut AnimationStateMachineAsset, state_name: &str) -> bool {
    if asset.entry_state == state_name || !has_state(asset, state_name) {
        return false;
    }
    asset.entry_state = state_name.to_string();
    true
}

pub(super) fn create_transition(
    asset: &mut AnimationStateMachineAsset,
    from_state: &str,
    to_state: &str,
    duration_frames: u32,
) -> bool {
    if !has_state(asset, from_state) || !has_state(asset, to_state) {
        return false;
    }
    let duration_seconds = frame_to_seconds(duration_frames);
    if let Some(transition) = asset
        .transitions
        .iter_mut()
        .find(|transition| transition.from_state == from_state && transition.to_state == to_state)
    {
        let changed = (transition.duration_seconds - duration_seconds).abs() > f32::EPSILON;
        transition.duration_seconds = duration_seconds;
        return changed;
    }
    asset.transitions.push(AnimationStateTransitionAsset {
        from_state: from_state.to_string(),
        to_state: to_state.to_string(),
        duration_seconds,
        exit_time: None,
        interruption: Default::default(),
        conditions: Vec::new(),
    });
    true
}

pub(super) fn remove_transition(
    asset: &mut AnimationStateMachineAsset,
    from_state: &str,
    to_state: &str,
) -> bool {
    let before = asset.transitions.len();
    asset.transitions.retain(|transition| {
        !(transition.from_state == from_state && transition.to_state == to_state)
    });
    before != asset.transitions.len()
}

pub(super) fn set_transition_condition(
    asset: &mut AnimationStateMachineAsset,
    from_state: &str,
    to_state: &str,
    parameter_name: &str,
    operator: &str,
    value_literal: &str,
) -> bool {
    if !has_state(asset, from_state) || !has_state(asset, to_state) {
        return false;
    }
    let Some(transition) = asset
        .transitions
        .iter_mut()
        .find(|transition| transition.from_state == from_state && transition.to_state == to_state)
    else {
        return false;
    };
    let Some(operator) = parse_condition_operator(operator) else {
        return false;
    };
    let existing_value = transition
        .conditions
        .iter()
        .find(|condition| condition.parameter == parameter_name)
        .and_then(|condition| condition.value.clone());
    let Some(value) = parse_parameter_value(existing_value.as_ref(), value_literal) else {
        return false;
    };
    let next_condition = AnimationTransitionConditionAsset {
        parameter: parameter_name.to_string(),
        operator,
        value: Some(value),
    };
    if let Some(condition) = transition
        .conditions
        .iter_mut()
        .find(|condition| condition.parameter == parameter_name)
    {
        let changed = *condition != next_condition;
        *condition = next_condition;
        return changed;
    }
    transition.conditions.push(next_condition);
    true
}

fn has_state(asset: &AnimationStateMachineAsset, state_name: &str) -> bool {
    asset.states.iter().any(|state| state.name == state_name)
}

fn frame_to_seconds(frame: u32) -> f32 {
    frame as f32 / TRANSITION_FRAMES_PER_SECOND
}

fn parse_condition_operator(operator: &str) -> Option<AnimationConditionOperatorAsset> {
    match operator {
        "equal" => Some(AnimationConditionOperatorAsset::Equal),
        "not_equal" => Some(AnimationConditionOperatorAsset::NotEqual),
        "greater" => Some(AnimationConditionOperatorAsset::Greater),
        "greater_equal" => Some(AnimationConditionOperatorAsset::GreaterEqual),
        "less" => Some(AnimationConditionOperatorAsset::Less),
        "less_equal" => Some(AnimationConditionOperatorAsset::LessEqual),
        "triggered" => Some(AnimationConditionOperatorAsset::Triggered),
        _ => None,
    }
}

fn parse_parameter_value(
    existing: Option<&AnimationParameterValue>,
    value_literal: &str,
) -> Option<AnimationParameterValue> {
    match existing {
        Some(AnimationParameterValue::Trigger) => parse_trigger_literal(value_literal),
        Some(AnimationParameterValue::Bool(_)) => {
            parse_bool_literal(value_literal).map(AnimationParameterValue::Bool)
        }
        Some(AnimationParameterValue::Integer(_)) => value_literal
            .parse::<i32>()
            .ok()
            .map(AnimationParameterValue::Integer),
        Some(AnimationParameterValue::Scalar(_)) => {
            parse_finite_scalar_literal(value_literal).map(AnimationParameterValue::Scalar)
        }
        Some(AnimationParameterValue::Vec2(_)) => {
            parse_vector_literal::<2>(value_literal).map(AnimationParameterValue::Vec2)
        }
        Some(AnimationParameterValue::Vec3(_)) => {
            parse_vector_literal::<3>(value_literal).map(AnimationParameterValue::Vec3)
        }
        Some(AnimationParameterValue::Vec4(_)) => {
            parse_vector_literal::<4>(value_literal).map(AnimationParameterValue::Vec4)
        }
        None => parse_trigger_literal(value_literal)
            .or_else(|| parse_bool_literal(value_literal).map(AnimationParameterValue::Bool))
            .or_else(|| {
                value_literal
                    .parse::<i32>()
                    .ok()
                    .map(AnimationParameterValue::Integer)
            })
            .or_else(|| {
                parse_finite_scalar_literal(value_literal).map(AnimationParameterValue::Scalar)
            })
            .or_else(|| parse_vector_literal::<2>(value_literal).map(AnimationParameterValue::Vec2))
            .or_else(|| parse_vector_literal::<3>(value_literal).map(AnimationParameterValue::Vec3))
            .or_else(|| {
                parse_vector_literal::<4>(value_literal).map(AnimationParameterValue::Vec4)
            }),
    }
}

fn parse_finite_scalar_literal(value_literal: &str) -> Option<f32> {
    let value = value_literal.parse::<f32>().ok()?;
    value.is_finite().then_some(value)
}

fn parse_trigger_literal(value_literal: &str) -> Option<AnimationParameterValue> {
    value_literal
        .eq_ignore_ascii_case("trigger")
        .then_some(AnimationParameterValue::Trigger)
}

fn parse_bool_literal(value_literal: &str) -> Option<bool> {
    if value_literal.eq_ignore_ascii_case("true") {
        Some(true)
    } else if value_literal.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

fn parse_vector_literal<const N: usize>(value_literal: &str) -> Option<[f32; N]> {
    let parts: Vec<_> = value_literal.split(',').map(str::trim).collect();
    if parts.len() != N {
        return None;
    }
    let mut values = [0.0; N];
    for (index, part) in parts.into_iter().enumerate() {
        values[index] = parse_finite_scalar_literal(part)?;
    }
    Some(values)
}
