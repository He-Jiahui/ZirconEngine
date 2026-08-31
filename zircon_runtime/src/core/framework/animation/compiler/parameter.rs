//! Shared parameter descriptors used by compiled animation products.

use crate::core::framework::animation::AnimationParameterValue;

/// Type contract declared or inferred by a compiled animation product parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationCompiledParameterKind {
    Bool,
    Integer,
    /// A parameter accepted by numeric transition comparisons as either an integer or scalar.
    Numeric,
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Trigger,
}

/// A named parameter retained in a compiled animation artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledParameter {
    name: String,
    kind: AnimationCompiledParameterKind,
    default_value: Option<AnimationParameterValue>,
}

impl AnimationCompiledParameter {
    pub(crate) fn with_default(name: String, default_value: AnimationParameterValue) -> Self {
        Self {
            kind: parameter_kind(&default_value),
            name,
            default_value: Some(default_value),
        }
    }

    pub(crate) fn declared(name: String, kind: AnimationCompiledParameterKind) -> Self {
        Self {
            name,
            kind,
            default_value: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> AnimationCompiledParameterKind {
        self.kind
    }

    pub(crate) fn set_kind(&mut self, kind: AnimationCompiledParameterKind) {
        self.kind = kind;
    }

    /// Returns the authored default when the source schema has one.
    pub fn default_value(&self) -> Option<&AnimationParameterValue> {
        self.default_value.as_ref()
    }
}

pub(crate) fn parameter_kind(value: &AnimationParameterValue) -> AnimationCompiledParameterKind {
    match value {
        AnimationParameterValue::Bool(_) => AnimationCompiledParameterKind::Bool,
        AnimationParameterValue::Integer(_) => AnimationCompiledParameterKind::Integer,
        AnimationParameterValue::Scalar(_) => AnimationCompiledParameterKind::Scalar,
        AnimationParameterValue::Vec2(_) => AnimationCompiledParameterKind::Vec2,
        AnimationParameterValue::Vec3(_) => AnimationCompiledParameterKind::Vec3,
        AnimationParameterValue::Vec4(_) => AnimationCompiledParameterKind::Vec4,
        AnimationParameterValue::Trigger => AnimationCompiledParameterKind::Trigger,
    }
}

pub(crate) fn parameter_value_is_finite(value: &AnimationParameterValue) -> bool {
    match value {
        AnimationParameterValue::Scalar(value) => value.is_finite(),
        AnimationParameterValue::Vec2(values) => values.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec3(values) => values.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec4(values) => values.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Bool(_)
        | AnimationParameterValue::Integer(_)
        | AnimationParameterValue::Trigger => true,
    }
}
