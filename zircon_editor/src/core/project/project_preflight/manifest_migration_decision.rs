use super::ProjectManifestMigrationPlan;

/// Records whether accepting a parsed manifest requires an explicit migration decision.
///
/// Parsing may migrate a supported older document in memory. The resulting receipt is never an
/// activation permit until an owning transaction chooses, executes, and re-preflights an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectManifestMigrationDecision {
    Current,
    RequiresExplicitDecision { plan: ProjectManifestMigrationPlan },
}

impl ProjectManifestMigrationDecision {
    pub(in crate::core::project) const fn from_migrated_from(migrated_from: Option<u32>) -> Self {
        match migrated_from {
            Some(source_format_version) => Self::RequiresExplicitDecision {
                plan: ProjectManifestMigrationPlan::from_source_format_version(
                    source_format_version,
                ),
            },
            None => Self::Current,
        }
    }

    /// A legacy receipt cannot enter activation until its selected action has produced a fresh
    /// preflight receipt for the resulting project state.
    pub const fn blocks_activation(self) -> bool {
        matches!(self, Self::RequiresExplicitDecision { .. })
    }
}
