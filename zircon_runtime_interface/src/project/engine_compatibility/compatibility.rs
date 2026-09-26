use serde::{Deserialize, Serialize};

use super::{ProjectEngineCompatibilityDisposition, ProjectEngineVersion};

/// 项目预检形成的版本判定快照；Editor 在激活前消费其兼容结论。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectEngineCompatibility {
    requirement: Option<String>,
    running_engine: ProjectEngineVersion,
    disposition: ProjectEngineCompatibilityDisposition,
}

impl ProjectEngineCompatibility {
    pub(crate) fn new(
        requirement: Option<String>,
        running_engine: ProjectEngineVersion,
        disposition: ProjectEngineCompatibilityDisposition,
    ) -> Self {
        Self {
            requirement,
            running_engine,
            disposition,
        }
    }

    pub fn requirement(&self) -> Option<&str> {
        self.requirement.as_deref()
    }

    pub fn running_engine(&self) -> &ProjectEngineVersion {
        &self.running_engine
    }

    pub const fn disposition(&self) -> ProjectEngineCompatibilityDisposition {
        self.disposition
    }

    /// 返回快照中的预检结论，不重新解析要求或读取当前引擎版本。
    pub const fn is_compatible(&self) -> bool {
        self.disposition.is_compatible()
    }
}
