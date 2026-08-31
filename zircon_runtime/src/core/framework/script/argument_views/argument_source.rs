use super::super::{ScriptHostError, ScriptHostValue};
use super::byte_view::ScriptHostByteView;
use super::value_ref::ScriptHostValueRef;

pub trait ScriptHostArgumentSource {
    fn len(&self) -> usize;

    fn visit_argument(
        &self,
        index: usize,
        visitor: &mut dyn for<'argument> FnMut(
            ScriptHostValueRef<'argument>,
        ) -> Result<(), ScriptHostError>,
    ) -> Result<(), ScriptHostError>;
}

pub struct ScriptHostArguments<'call> {
    source: &'call dyn ScriptHostArgumentSource,
}

impl<'call> ScriptHostArguments<'call> {
    pub fn new(source: &'call dyn ScriptHostArgumentSource) -> Self {
        Self { source }
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with_argument<T>(
        &self,
        index: usize,
        visitor: impl for<'argument> FnOnce(ScriptHostValueRef<'argument>) -> Result<T, ScriptHostError>,
    ) -> Result<T, ScriptHostError> {
        let mut visitor = Some(visitor);
        let mut result = None;
        self.source.visit_argument(index, &mut |value| {
            let visitor = visitor.take().ok_or_else(|| {
                ScriptHostError::new("script host argument visitor was invoked more than once")
            })?;
            result = Some(visitor(value));
            Ok(())
        })?;
        result.unwrap_or_else(|| {
            Err(ScriptHostError::new(
                "script host argument visitor completed without a result",
            ))
        })
    }
}

pub(crate) struct ScriptHostOwnedArgumentSource<'call> {
    values: &'call [ScriptHostValue],
}

impl<'call> ScriptHostOwnedArgumentSource<'call> {
    pub(crate) fn new(values: &'call [ScriptHostValue]) -> Self {
        Self { values }
    }
}

impl ScriptHostArgumentSource for ScriptHostOwnedArgumentSource<'_> {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn visit_argument(
        &self,
        index: usize,
        visitor: &mut dyn for<'argument> FnMut(
            ScriptHostValueRef<'argument>,
        ) -> Result<(), ScriptHostError>,
    ) -> Result<(), ScriptHostError> {
        let value = self
            .values
            .get(index)
            .ok_or_else(|| ScriptHostError::new(format!("argument {index} was not provided")))?;
        visitor(script_host_value_ref(value))
    }
}

fn script_host_value_ref(value: &ScriptHostValue) -> ScriptHostValueRef<'_> {
    match value {
        ScriptHostValue::Null => ScriptHostValueRef::Null,
        ScriptHostValue::Bool(value) => ScriptHostValueRef::Bool(*value),
        ScriptHostValue::Int(value) => ScriptHostValueRef::Int(*value),
        ScriptHostValue::Float(value) => ScriptHostValueRef::Float(*value),
        ScriptHostValue::String(value) => ScriptHostValueRef::String(value),
        ScriptHostValue::Bytes(value) => {
            ScriptHostValueRef::Bytes(ScriptHostByteView::Slice(value))
        }
        ScriptHostValue::HostHandle(value) => ScriptHostValueRef::HostHandle(*value),
    }
}
