use zircon_runtime_interface::project::ProjectLaunchIntent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorGuiStartupRequest {
    Project { intent: ProjectLaunchIntent },
    OpenBuiltinView { descriptor_id: String },
}

impl EditorGuiStartupRequest {
    /// Carries a versioned project operation into the host without assigning it an identity.
    pub fn project(intent: ProjectLaunchIntent) -> Self {
        Self::Project { intent }
    }

    pub fn open_builtin_view(descriptor_id: impl Into<String>) -> Self {
        Self::OpenBuiltinView {
            descriptor_id: descriptor_id.into(),
        }
    }

    pub fn project_intent(&self) -> Option<&ProjectLaunchIntent> {
        match self {
            Self::Project { intent } => Some(intent),
            Self::OpenBuiltinView { .. } => None,
        }
    }
}
