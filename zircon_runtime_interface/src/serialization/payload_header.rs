use serde::{Deserialize, Serialize};

use super::SchemaId;

/// Header shared by every versioned text or binary envelope.
/// 文本和二进制版本封装共用的 schema 身份与版本头；版本化读取端先匹配 schema 并拒绝未来版本，再解码或迁移载荷；无头 v0 回退由显式 legacy loader 控制。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadHeader {
    pub schema_id: SchemaId,
    pub schema_version: u32,
}
