use thiserror::Error;

/// Failure to construct a canonical persistent asset reference.
/// 可选 sub 路径为空、含片段分隔符或控制字符时构造与反序列化都会失败。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AssetRefError {
    #[error("asset subpath cannot be empty")]
    EmptySubPath,
    #[error("asset subpath cannot contain the fragment delimiter '#'")]
    FragmentDelimiterInSubPath,
    #[error("asset subpath cannot contain a control character at byte index {index}")]
    ControlCharacterInSubPath { index: usize },
}
