//! 聚合 BinaryValue 节点表示、值域转换和直接目标类型反序列化。
//! 旧 schema 经 JSON 值进入迁移链；当前 schema 直接从节点流恢复目标类型，两条读取路径都执行值域校验。

#[path = "binary_value.rs"]
mod binary_value;
#[path = "direct_decode.rs"]
mod direct_decode;
#[path = "error.rs"]
mod error;
#[path = "from_json.rs"]
mod from_json;
#[path = "into_json.rs"]
mod into_json;

pub(in crate::serialization) use binary_value::{BinaryNode, BinaryValue};
pub(in crate::serialization) use direct_decode::{
    decode_binary_value_direct, DirectBinaryDecodeError,
};
pub(in crate::serialization) use error::BinaryValueError;
