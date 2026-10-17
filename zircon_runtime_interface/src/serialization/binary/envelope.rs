//! 二进制 body 的有序封套；固定 wire 前缀在此结构之外单独解析。

use serde::{Deserialize, Serialize};

use crate::serialization::PayloadHeader;

use super::value::BinaryValue;

/// Bincode-owned body following the fixed binary wire prefix.
///
/// Field order is part of wire v1. Decode reads and validates `header` before
/// deserializing `payload`; changing this order requires a wire-version bump.
///
/// `header` carries the payload's schema identity and schema version; the prefix's wire version is
/// separate. Header parsing happens in `decode_binary_header`, while target-schema and supported-
/// version checks belong to `load_binary_versioned` before that caller decodes the payload.
///
/// # 使用时机
/// The binary writer serializes this envelope after canonicalizing the payload into a bounded
/// `BinaryValue` node stream. The loader uses the same field order for current typed values and
/// older values that must enter the migration path.
/// 该结构序列化为前缀之后的完整 bincode body；字段声明顺序属于 wire v1 的持久化布局。
#[derive(Deserialize, Serialize)]
pub(super) struct BinaryEnvelope {
    /// Domain schema identity and version, checked against the requested `VersionedSchema` by the loader.
    /// loader 在读取 payload 前用它验证目标 schema 身份及可支持的 schema 版本。
    pub(super) header: PayloadHeader,
    /// The payload's flat binary value stream; it follows the header in the wire body.
    /// 当前版本在校验头部后直接解码为目标类型；旧版本则转为迁移链使用的 JSON 值。
    pub(super) payload: BinaryValue,
}
