use thiserror::Error;

/// Reports malformed manifest-digest wire text before it reaches a session boundary.
/// 记录摘要十六进制文本不符合长度或字符规范时的解析错误。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error(
    "project manifest digest must be exactly {expected_hex_characters} lowercase hexadecimal characters, received `{value}`"
)]
pub struct ProjectManifestDigestParseError {
    /// 解析失败时收到的原始文本，保留给诊断信息使用。
    pub value: String,
    expected_hex_characters: usize,
}

impl ProjectManifestDigestParseError {
    pub(crate) fn new(value: String, expected_hex_characters: usize) -> Self {
        Self {
            value,
            expected_hex_characters,
        }
    }
}
