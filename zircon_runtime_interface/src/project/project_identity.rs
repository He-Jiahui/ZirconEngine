use serde::{Deserialize, Serialize};

use super::{CanonicalDescriptorIdentity, ProjectGuid, ProjectManifestDigest};

/// Versioned project identity accepted by preflight and later admission boundaries.
/// 由物理 descriptor 路径、持久项目 GUID 和本次清单摘要组成的预检身份。
/// 后续准入不能只凭路径或 GUID 沿用旧收据；清单字节变化须重新预检。
/// Serde 解码只应用各组成类型的表示约束，不读取路径或核对摘要与当前文件是否相符。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIdentity {
    canonical_descriptor: CanonicalDescriptorIdentity,
    project_guid: ProjectGuid,
    manifest_digest: ProjectManifestDigest,
}

impl ProjectIdentity {
    /// 组合调用方提供的路径身份、项目 GUID 与清单摘要，不执行文件系统或内容核验。
    ///
    /// # 参数
    /// - `canonical_descriptor`：描述符路径身份值；其物理解析由调用方边界负责。
    /// - `project_guid`：已通过非 nil 约束的持久项目标识。
    /// - `manifest_digest`：调用方按清单原始字节计算的摘要。
    ///
    /// # 返回
    /// 保存这三项值的身份快照；不验证它们来自同一项目或仍对应当前文件。
    pub fn new(
        canonical_descriptor: CanonicalDescriptorIdentity,
        project_guid: ProjectGuid,
        manifest_digest: ProjectManifestDigest,
    ) -> Self {
        Self {
            canonical_descriptor,
            project_guid,
            manifest_digest,
        }
    }

    pub fn canonical_descriptor(&self) -> &CanonicalDescriptorIdentity {
        &self.canonical_descriptor
    }

    pub const fn project_guid(&self) -> ProjectGuid {
        self.project_guid
    }

    pub const fn manifest_digest(&self) -> ProjectManifestDigest {
        self.manifest_digest
    }
}
