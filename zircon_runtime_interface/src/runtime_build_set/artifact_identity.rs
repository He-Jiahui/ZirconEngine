use serde::{Deserialize, Serialize};

use super::{ZrRuntimeDigestV1, ZrRuntimeIdentityFormatError};

/// Names and hashes the exact dynamic library selected by a release manifest.
/// 标识发布清单选中的确切动态库文件名及其 SHA-256 摘要。
/// `file_name` 只能是单个文件名，实际文件内容的哈希由加载前调用方独立计算。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZrRuntimeArtifactIdentityV1 {
    pub file_name: String,
    pub sha256: ZrRuntimeDigestV1,
}

impl ZrRuntimeArtifactIdentityV1 {
    /// 构造并立即校验文件名；摘要只按值保存，不在此处读取文件。
    pub fn new(
        file_name: impl Into<String>,
        sha256: ZrRuntimeDigestV1,
    ) -> Result<Self, ZrRuntimeIdentityFormatError> {
        let identity = Self {
            file_name: file_name.into(),
            sha256,
        };
        identity.validate()?;
        Ok(identity)
    }

    /// 只接受文件名而不接受目录路径；App 另对实际 staged 文件计算哈希并比对。
    /// 这里仅拒绝空名、`.`、`..` 和两种路径分隔符；加载端另行核对实际路径及文件内容。
    pub fn validate(&self) -> Result<(), ZrRuntimeIdentityFormatError> {
        let name = self.file_name.as_str();
        if name.is_empty() || matches!(name, "." | "..") || name.contains(['/', '\\']) {
            return Err(ZrRuntimeIdentityFormatError::ArtifactFileName {
                value: self.file_name.clone(),
            });
        }
        Ok(())
    }
}
