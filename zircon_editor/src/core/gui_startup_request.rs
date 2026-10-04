use zircon_runtime_interface::project::ProjectLaunchIntent;

use super::project::{ProjectAuthority, ProjectAuthorityError, ProjectLaunchPreflight};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorGuiStartupRequest {
    Project {
        intent: ProjectLaunchIntent,
        preflight: Option<ProjectLaunchPreflight>,
    },
    OpenBuiltinView {
        descriptor_id: String,
    },
}

impl EditorGuiStartupRequest {
    /// Carries a versioned project operation into the host without assigning it an identity.
    pub fn project(intent: ProjectLaunchIntent) -> Self {
        Self::Project {
            intent,
            preflight: None,
        }
    }

    pub fn open_builtin_view(descriptor_id: impl Into<String>) -> Self {
        Self::OpenBuiltinView {
            descriptor_id: descriptor_id.into(),
        }
    }

    pub fn project_intent(&self) -> Option<&ProjectLaunchIntent> {
        match self {
            Self::Project { intent, .. } => Some(intent),
            Self::OpenBuiltinView { .. } => None,
        }
    }

    /// Freezes project-derived composition input before App creates the product Core.
    pub fn preflight_project(
        self,
        authority: &ProjectAuthority,
    ) -> Result<Self, ProjectAuthorityError> {
        match self {
            Self::Project { intent, preflight } => {
                let preflight = match preflight {
                    Some(preflight) => preflight,
                    None => authority.preflight_project_launch(intent.clone())?,
                };
                Ok(Self::Project {
                    intent,
                    preflight: Some(preflight),
                })
            }
            request @ Self::OpenBuiltinView { .. } => Ok(request),
        }
    }

    pub fn project_preflight(&self) -> Option<&ProjectLaunchPreflight> {
        match self {
            Self::Project { preflight, .. } => preflight.as_ref(),
            Self::OpenBuiltinView { .. } => None,
        }
    }
}
