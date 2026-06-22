use zircon_runtime::core::framework::animation::AnimationParameterValue;

pub(super) fn parameter_value_label(value: &AnimationParameterValue) -> String {
    match value {
        AnimationParameterValue::Bool(value) => value.to_string(),
        AnimationParameterValue::Integer(value) => value.to_string(),
        AnimationParameterValue::Scalar(value) => format!("{value:.2}"),
        AnimationParameterValue::Vec2(value) => format!("{}, {}", value[0], value[1]),
        AnimationParameterValue::Vec3(value) => {
            format!("{}, {}, {}", value[0], value[1], value[2])
        }
        AnimationParameterValue::Vec4(value) => {
            format!("{}, {}, {}, {}", value[0], value[1], value[2], value[3])
        }
        AnimationParameterValue::Trigger => "trigger".to_string(),
    }
}

pub(super) fn parse_parameter_value(
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
