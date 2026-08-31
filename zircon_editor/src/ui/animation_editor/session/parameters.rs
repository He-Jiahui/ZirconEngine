use zircon_runtime::core::framework::animation::AnimationParameterValue;

pub(super) fn parameter_value_label(value: &AnimationParameterValue) -> String {
    match value {
        AnimationParameterValue::Bool(value) => value.to_string(),
        AnimationParameterValue::Integer(value) => value.to_string(),
        AnimationParameterValue::Scalar(value) => format!("{value:.2}"),
        AnimationParameterValue::Vec2(value) => format!("{}, {}", value[0], value[1]),
        AnimationParameterValue::Vec3(value) => format!("{}, {}, {}", value[0], value[1], value[2]),
        AnimationParameterValue::Vec4(value) => {
            format!("{}, {}, {}, {}", value[0], value[1], value[2], value[3])
        }
        AnimationParameterValue::Trigger => "trigger".to_string(),
    }
}
