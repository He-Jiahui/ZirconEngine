use serde::{Deserialize, Serialize};

/// Whether the selected engine satisfies a project's declared semantic-version range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEngineCompatibilityDisposition {
    Compatible,
    ProjectRequiresNewerEngine,
    ProjectRequiresOlderEngine,
    Incompatible,
}

impl ProjectEngineCompatibilityDisposition {
    pub const fn is_compatible(self) -> bool {
        matches!(self, Self::Compatible)
    }
}
