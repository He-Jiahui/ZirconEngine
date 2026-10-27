//! Gameplay essentials extend the same capability-checked module descriptor as existing exports.
use crate::core::framework::script::{
    ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor, ScriptHostParameterDescriptor,
    ScriptHostValueKind,
};

pub(super) fn with_essentials(
    descriptor: ScriptHostModuleDescriptor,
) -> ScriptHostModuleDescriptor {
    descriptor
        .with_function(
            function("key_pressed", 1, 1, ScriptHostValueKind::Bool)
                .with_parameter(string_parameter("key"))
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("random_range", 2, 2, ScriptHostValueKind::Float)
                .with_parameter(float_parameter("min"))
                .with_parameter(float_parameter("max"))
                .with_required_capability("gameplay.random"),
        )
        .with_function(
            function("mouse_pressed", 1, 1, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("button"))
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("set_rotation_euler", 4, 4, ScriptHostValueKind::Bool)
                .with_parameter(int_parameter("entity"))
                .with_parameter(float_parameter("x"))
                .with_parameter(float_parameter("y"))
                .with_parameter(float_parameter("z"))
                .with_required_capability("gameplay.entity")
                .with_documentation("Set world rotation from XYZ Euler angles in radians; preserve position and scale."),
        )
        .with_function(
            function("component_number", 3, 3, ScriptHostValueKind::Float)
                .with_parameter(int_parameter("entity"))
                .with_parameter(string_parameter("component_id"))
                .with_parameter(float_parameter("fallback"))
                .with_required_capability("gameplay.entity"),
        )
        .with_function(
            function("mouse_x", 0, 0, ScriptHostValueKind::Float)
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("mouse_y", 0, 0, ScriptHostValueKind::Float)
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("mouse_delta_x", 0, 0, ScriptHostValueKind::Float)
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("mouse_delta_y", 0, 0, ScriptHostValueKind::Float)
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("mouse_wheel_x", 0, 0, ScriptHostValueKind::Float)
                .with_required_capability("gameplay.input"),
        )
        .with_function(
            function("mouse_wheel_y", 0, 0, ScriptHostValueKind::Float)
                .with_required_capability("gameplay.input"),
        )
}

pub(super) fn function(
    name: impl Into<String>,
    min_argument_count: usize,
    max_argument_count: usize,
    return_value_kind: ScriptHostValueKind,
) -> ScriptHostFunctionDescriptor {
    ScriptHostFunctionDescriptor::new(
        name,
        min_argument_count,
        max_argument_count,
        return_value_kind,
    )
}

pub(super) fn string_parameter(name: impl Into<String>) -> ScriptHostParameterDescriptor {
    ScriptHostParameterDescriptor::new(name, ScriptHostValueKind::String)
}

pub(super) fn int_parameter(name: impl Into<String>) -> ScriptHostParameterDescriptor {
    ScriptHostParameterDescriptor::new(name, ScriptHostValueKind::Int)
}

pub(super) fn float_parameter(name: impl Into<String>) -> ScriptHostParameterDescriptor {
    ScriptHostParameterDescriptor::new(name, ScriptHostValueKind::Float)
}

pub(super) fn bool_parameter(name: impl Into<String>) -> ScriptHostParameterDescriptor {
    ScriptHostParameterDescriptor::new(name, ScriptHostValueKind::Bool)
}
