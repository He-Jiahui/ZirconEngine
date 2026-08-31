use super::super::{ScriptHostError, ScriptHostTypeRef, ScriptHostValueKind};
use super::value_ref::ScriptHostValueRef;

pub trait ScriptHostFromArgument: Sized {
    fn script_host_type_ref() -> ScriptHostTypeRef;

    fn from_script_host_argument(
        value: ScriptHostValueRef<'_>,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError>;
}

impl ScriptHostFromArgument for bool {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bool)
    }

    fn from_script_host_argument(
        value: ScriptHostValueRef<'_>,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValueRef::Bool(value) => Ok(value),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Bool,
                value.kind(),
            )),
        }
    }
}

impl ScriptHostFromArgument for i64 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Int)
    }

    fn from_script_host_argument(
        value: ScriptHostValueRef<'_>,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValueRef::Int(value) => Ok(value),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Int,
                value.kind(),
            )),
        }
    }
}

impl ScriptHostFromArgument for f64 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Float)
    }

    fn from_script_host_argument(
        value: ScriptHostValueRef<'_>,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValueRef::Float(value) => Ok(value),
            ScriptHostValueRef::Int(value) => Ok(value as f64),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Float,
                value.kind(),
            )),
        }
    }
}

impl ScriptHostFromArgument for f32 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Float)
    }

    fn from_script_host_argument(
        value: ScriptHostValueRef<'_>,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        f64::from_script_host_argument(value, argument_index).map(|value| value as f32)
    }
}

pub(super) fn argument_type_error(
    argument_index: usize,
    expected: ScriptHostValueKind,
    actual: ScriptHostValueKind,
) -> ScriptHostError {
    ScriptHostError::new(format!(
        "argument {argument_index} expected {expected:?}, received {actual:?}"
    ))
}
