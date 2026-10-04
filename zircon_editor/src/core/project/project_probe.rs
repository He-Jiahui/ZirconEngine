//! 把只读探测得到的清单摘要和物理身份交给欢迎页后台检查及最近项目登记；结果不持有已打开项目或激活能力。
use std::path::Path;

use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime_interface::project::ProjectManifestSummary;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectProbe {
    identity: ResolvedProjectPath,
    summary: ProjectManifestSummary,
}

impl ProjectProbe {
    pub(super) fn new(identity: ResolvedProjectPath, summary: ProjectManifestSummary) -> Self {
        Self { identity, summary }
    }

    pub fn root(&self) -> &Path {
        self.identity.operation_path()
    }

    pub fn identity(&self) -> &ResolvedProjectPath {
        &self.identity
    }

    pub fn summary(&self) -> &ProjectManifestSummary {
        &self.summary
    }
}
