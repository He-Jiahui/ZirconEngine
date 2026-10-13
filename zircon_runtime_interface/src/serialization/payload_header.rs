use serde::{Deserialize, Serialize};

use super::SchemaId;

/// Header shared by every versioned text or binary envelope.
/// 文本和二进制版本封装共用的 schema 身份与版本头；版本化读取端先匹配 schema 并拒绝未来版本，再解码或迁移载荷；无头 v0 回退由显式 legacy loader 控制。
///
/// # 约束
/// `deny_unknown_fields` 使封装中的头部字段必须与当前 wire 契约一致；`schema_id` 标识
/// 载荷族，`schema_version` 标识该族内的数据形状版本。
///
/// # 参考
/// 由 [`VersionedSchema`](super::VersionedSchema) 提供目标身份和当前版本，并由版本化 loader
/// 在解析 payload 之前检查。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadHeader {
    /// 该载荷所属的稳定 schema 身份。
    pub schema_id: SchemaId,
    /// 写入该载荷时使用的 schema 版本。
    pub schema_version: u32,
}
