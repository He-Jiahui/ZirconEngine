use super::ProjectManifestMigrationAction;

/// Static migration policy produced from a supported legacy manifest version.
///
/// A selected action must be executed by the owning transaction and followed by a fresh
/// preflight. This plan never grants activation permission for the legacy receipt itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectManifestMigrationPlan {
    source_format_version: u32,
}

impl ProjectManifestMigrationPlan {
    pub(in crate::core::project) const fn from_source_format_version(
        source_format_version: u32,
    ) -> Self {
        Self {
            source_format_version,
        }
    }

    pub const fn source_format_version(self) -> u32 {
        self.source_format_version
    }

    pub const fn available_actions(self) -> [ProjectManifestMigrationAction; 3] {
        ProjectManifestMigrationAction::ALL
    }
}
