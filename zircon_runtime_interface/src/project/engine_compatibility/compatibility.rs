use serde::{Deserialize, Serialize};

use super::{ProjectEngineCompatibilityDisposition, ProjectEngineVersion};

/// Immutable engine-version decision that an admission policy must consume before activation.
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

    pub const fn is_compatible(&self) -> bool {
        self.disposition.is_compatible()
    }
}
