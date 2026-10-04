use super::super::{
    ScriptHostError, ScriptHostHandleValue, ScriptHostHotPathMetrics, ScriptHostValueKind,
};
use super::byte_view::ScriptHostByteView;
use super::typed_conversion::argument_type_error;

/// 借用型参数值，只在 ScriptHostArguments 的同步 visitor 中有效。
///
/// 字符串和字节需要跨越调用栈时，应在明确的业务边界复制并计入热路径指标。
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

    /// 将借用字符串转为业务拥有值，并记录这次跨边界复制的字节数。
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

    /// 将借用字节转为业务拥有值；guest 源的读取错误会原样返回。
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
