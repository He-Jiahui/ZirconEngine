//! 带固定前缀和版本头的二进制载荷实现。
//!
//! 这里把 wire 前缀、bincode 封套和 `BinaryValue` 节点载荷拼成既有 v1 字节格式。写入由
//! [`crate::serialization::write_versioned`] 在选择 `Format::Binary` 时进入；读取时，
//! [`crate::serialization::load_versioned`] 先取得封套头并检查目标 schema/version，再调用
//! 对应 payload 解码入口。前缀 wire version 与头部 schema version 属于不同兼容边界。

#[path = "decode.rs"]
mod decode;
#[path = "encode.rs"]
mod encode;
#[path = "envelope.rs"]
mod envelope;
#[path = "value/mod.rs"]
mod value;
#[path = "wire.rs"]
mod wire;

pub(super) use decode::{decode_binary_current, decode_binary_header, decode_binary_payload};
pub(super) use encode::encode_binary_payload;
#[cfg(test)]
pub(super) use encode::encode_binary_value;
pub(super) use value::DirectBinaryDecodeError;
#[cfg(test)]
pub(super) use value::{BinaryNode, BinaryValue};
#[cfg(test)]
pub(super) use wire::{MAX_BINARY_BODY_BYTES, MAX_BINARY_DEPTH, MAX_BINARY_STRING_BYTES};
