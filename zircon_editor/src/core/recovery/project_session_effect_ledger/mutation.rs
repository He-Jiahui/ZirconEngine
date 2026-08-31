use super::ProjectSessionEffectDisposition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProjectSessionEffectMutation {
    Prepare,
    Commit,
    RollBack,
    RequireRecovery,
}

impl ProjectSessionEffectMutation {
    pub(super) const fn target(self) -> ProjectSessionEffectDisposition {
        match self {
            Self::Prepare => ProjectSessionEffectDisposition::Prepared,
            Self::Commit => ProjectSessionEffectDisposition::Committed,
            Self::RollBack => ProjectSessionEffectDisposition::RolledBack,
            Self::RequireRecovery => ProjectSessionEffectDisposition::RecoveryRequired,
        }
    }

    pub(super) const fn permits(self, current: Option<ProjectSessionEffectDisposition>) -> bool {
        match (current, self) {
            (None, Self::Prepare) => true,
            (Some(ProjectSessionEffectDisposition::Prepared), Self::Commit)
            | (Some(ProjectSessionEffectDisposition::Prepared), Self::RollBack)
            | (Some(ProjectSessionEffectDisposition::Prepared), Self::RequireRecovery)
            | (Some(ProjectSessionEffectDisposition::Committed), Self::RollBack)
            | (Some(ProjectSessionEffectDisposition::Committed), Self::RequireRecovery) => true,
            _ => false,
        }
    }
}
