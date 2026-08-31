use super::super::{
    ScriptHostError, ScriptHostHandleValue, ScriptHostHotPathMetrics, ScriptHostValueKind,
};
use super::byte_view::ScriptHostByteView;
use super::typed_conversion::argument_type_error;

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
