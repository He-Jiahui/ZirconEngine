use serde::{Deserialize, Serialize};

/// 槽位随档案持久化的来源、展示、时间和标签信息；Level 还原只映射项目与展示字段。
/// 标签在档案入口规范化，时间戳缺失在最新/最旧排序中按零处理。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionMetadata {
    #[serde(default)]
    pub project_root: Option<String>,
    #[serde(default)]
    pub asset_uri: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub created_at_unix_millis: Option<u64>,
    #[serde(default)]
    pub updated_at_unix_millis: Option<u64>,
    #[serde(default)]
    pub tags: Vec<String>,
}
