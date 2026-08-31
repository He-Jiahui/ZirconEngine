use serde::{Deserialize, Serialize};

use super::ProjectSessionEffectLedgerPhase;

/// A project-session side effect whose ownership must survive process termination.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectSessionEffect {
    Runtime,
    Diagnostics,
    ProjectPlugins,
    Documents,
    UserInterface,
    Session,
    RecentProjection,
    DirtyDocuments,
    AssetJobs,
    Play,
    FocusBinding,
    WorkspaceProjection,
}

impl ProjectSessionEffect {
    pub(crate) const ACTIVATION_EFFECTS: [Self; 6] = [
        Self::Runtime,
        Self::Diagnostics,
        Self::ProjectPlugins,
        Self::Documents,
        Self::UserInterface,
        Self::Session,
    ];

    pub(crate) const READY_EFFECTS: [Self; 1] = [Self::RecentProjection];

    /// Reverse-dependency close order. The session lease is always the final owner.
    pub(crate) const CLOSE_EFFECTS: [Self; 11] = [
        Self::DirtyDocuments,
        Self::FocusBinding,
        Self::AssetJobs,
        Self::UserInterface,
        Self::Play,
        Self::ProjectPlugins,
        Self::Runtime,
        Self::Documents,
        Self::Diagnostics,
        Self::WorkspaceProjection,
        Self::Session,
    ];

    pub(crate) fn allowed_in(self, phase: ProjectSessionEffectLedgerPhase) -> bool {
        let candidates = match phase {
            ProjectSessionEffectLedgerPhase::Activating => &Self::ACTIVATION_EFFECTS[..],
            ProjectSessionEffectLedgerPhase::Ready => &Self::READY_EFFECTS[..],
            ProjectSessionEffectLedgerPhase::Closing => &Self::CLOSE_EFFECTS[..],
            ProjectSessionEffectLedgerPhase::Closed
            | ProjectSessionEffectLedgerPhase::RecoveryRequired => &[],
        };
        candidates.contains(&self)
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Diagnostics => "diagnostics",
            Self::ProjectPlugins => "project_plugins",
            Self::Documents => "documents",
            Self::UserInterface => "user_interface",
            Self::Session => "session",
            Self::RecentProjection => "recent_projection",
            Self::DirtyDocuments => "dirty_documents",
            Self::AssetJobs => "asset_jobs",
            Self::Play => "play",
            Self::FocusBinding => "focus_binding",
            Self::WorkspaceProjection => "workspace_projection",
        }
    }
}
