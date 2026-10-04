//! 收据声明工件的内容身份。
//! 捕获层以已打开文件构造逻辑名、相对落盘路径、类别、摘要与字节数；后续 materialization 对相同字段验收。

use serde::{Deserialize, Serialize};

use super::file_digest::{digest_open_file, digest_open_file_with_buffer, FileDigestBuffer};
use super::{ArtifactKind, ProductReceiptError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 已打开产物的内容与目标落盘身份；验收时两者都必须与实际工件一致。
pub struct ReceiptArtifact {
    pub logical_name: String,
    pub relative_path: String,
    pub kind: ArtifactKind,
    pub sha256: String,
    pub byte_length: u64,
}

impl ReceiptArtifact {
    // Path resolution belongs to the platform-specific build owner; hash exactly this opened handle.
    pub fn capture_from_file(
        logical_name: impl Into<String>,
        relative_path: impl Into<String>,
        kind: ArtifactKind,
        file: std::fs::File,
    ) -> Result<Self, ProductReceiptError> {
        let digest = digest_open_file(file)?;

        Ok(Self {
            logical_name: logical_name.into(),
            relative_path: relative_path.into(),
            kind,
            sha256: digest.sha256,
            byte_length: digest.byte_length,
        })
    }

    pub(crate) fn capture_from_file_with_buffer(
        logical_name: impl Into<String>,
        relative_path: impl Into<String>,
        kind: ArtifactKind,
        file: std::fs::File,
        buffer: &mut FileDigestBuffer,
    ) -> Result<Self, ProductReceiptError> {
        let digest = digest_open_file_with_buffer(file, buffer)?;

        Ok(Self {
            logical_name: logical_name.into(),
            relative_path: relative_path.into(),
            kind,
            sha256: digest.sha256,
            byte_length: digest.byte_length,
        })
    }
}
