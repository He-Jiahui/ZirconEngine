use super::{
    ScriptHostError, ScriptHostHandleValue, ScriptHostHotPathMetrics, ScriptHostTypeRef,
    ScriptHostValue, ScriptHostValueKind,
};

pub trait ScriptHostByteSource {
    fn len(&self) -> Result<usize, ScriptHostError>;

    fn byte_at(&self, index: usize) -> Result<u8, ScriptHostError>;
}

#[derive(Clone, Copy)]
pub enum ScriptHostByteView<'call> {
    Slice(&'call [u8]),
    Source(&'call dyn ScriptHostByteSource),
}

impl ScriptHostByteView<'_> {
    pub fn len(&self) -> Result<usize, ScriptHostError> {
        match self {
            Self::Slice(bytes) => Ok(bytes.len()),
            Self::Source(source) => source.len(),
        }
    }

    pub fn byte_at(&self, index: usize) -> Result<u8, ScriptHostError> {
        match self {
            Self::Slice(bytes) => bytes.get(index).copied().ok_or_else(|| {
                ScriptHostError::new(format!("byte argument index {index} was not provided"))
            }),
            Self::Source(source) => source.byte_at(index),
        }
    }

    pub fn copy_to_vec_at_business_boundary(&self) -> Result<Vec<u8>, ScriptHostError> {
        let length = self.len()?;
        let mut bytes = Vec::with_capacity(length);
        for index in 0..length {
            bytes.push(self.byte_at(index)?);
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy)]
pub enum ScriptHostValueRef<'call> {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(&'call str),
    Bytes(ScriptHostByteView<'call>),
    HostHandle(ScriptHostHandleValue),
}

impl ScriptHostValueRef<'_> {
    pub fn kind(&self) -> ScriptHostValueKind {
        match self {
            Self::Null => ScriptHostValueKind::Null,
            Self::Bool(_) => ScriptHostValueKind::Bool,
            Self::Int(_) => ScriptHostValueKind::Int,
            Self::Float(_) => ScriptHostValueKind::Float,
            Self::String(_) => ScriptHostValueKind::String,
            Self::Bytes(_) => ScriptHostValueKind::Bytes,
            Self::HostHandle(_) => ScriptHostValueKind::HostHandle,
        }
    }

    pub fn copy_string_at_business_boundary(
        &self,
        argument_index: usize,
    ) -> Result<String, ScriptHostError> {
        match self {
            Self::String(value) => {
                ScriptHostHotPathMetrics::record_guest_string_copy(value.len());
                Ok((*value).to_owned())
            }
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::String,
                value.kind(),
            )),
        }
    }

    pub fn copy_bytes_at_business_boundary(
        &self,
        argument_index: usize,
    ) -> Result<Vec<u8>, ScriptHostError> {
        match self {
            Self::Bytes(value) => {
                let byte_count = value.len()?;
                ScriptHostHotPathMetrics::record_guest_byte_copy(byte_count);
                value.copy_to_vec_at_business_boundary()
            }
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Bytes,
                value.kind(),
            )),
        }
    }
}

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

fn argument_type_error(
    argument_index: usize,
    expected: ScriptHostValueKind,
    actual: ScriptHostValueKind,
) -> ScriptHostError {
    ScriptHostError::new(format!(
        "argument {argument_index} expected {expected:?}, received {actual:?}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_argument_source_lends_text_and_bytes_without_generic_transport_clones() {
        let values = [
            ScriptHostValue::String("borrowed".to_string()),
            ScriptHostValue::Bytes(vec![7, 0, 128, 255]),
        ];
        let source = ScriptHostOwnedArgumentSource::new(&values);
        let arguments = ScriptHostArguments::new(&source);

        let text_length = arguments
            .with_argument(0, |value| match value {
                ScriptHostValueRef::String(value) => Ok(value.len()),
                value => Err(ScriptHostError::new(format!(
                    "unexpected {:?}",
                    value.kind()
                ))),
            })
            .unwrap();
        assert_eq!(text_length, "borrowed".len());

        let checksum = arguments
            .with_argument(1, |value| match value {
                ScriptHostValueRef::Bytes(value) => {
                    let mut checksum = 0u32;
                    for index in 0..value.len()? {
                        checksum += u32::from(value.byte_at(index)?);
                    }
                    Ok(checksum)
                }
                value => Err(ScriptHostError::new(format!(
                    "unexpected {:?}",
                    value.kind()
                ))),
            })
            .unwrap();
        assert_eq!(checksum, 390);
    }

    #[test]
    fn explicit_owned_argument_conversions_record_only_their_business_boundary_copies() {
        let before = ScriptHostHotPathMetrics::snapshot();

        let text = ScriptHostValueRef::String("copy-at-boundary")
            .copy_string_at_business_boundary(0)
            .expect("string conversion should be valid");
        let bytes = ScriptHostValueRef::Bytes(ScriptHostByteView::Slice(&[7, 0, 128, 255]))
            .copy_bytes_at_business_boundary(1)
            .expect("byte conversion should be valid");

        let after = ScriptHostHotPathMetrics::snapshot();
        assert_eq!(text, "copy-at-boundary");
        assert_eq!(bytes, vec![7, 0, 128, 255]);
        assert!(
            after.guest_string_copy_bytes >= before.guest_string_copy_bytes + text.len() as u64
        );
        assert!(after.guest_byte_copy_bytes >= before.guest_byte_copy_bytes + bytes.len() as u64);
    }
}
