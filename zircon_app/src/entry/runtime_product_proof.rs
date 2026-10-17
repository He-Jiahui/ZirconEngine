//! Product proof policy shared by runtime and headless startup parsing.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::entry) enum RuntimeProductProofPolicy {
    #[default]
    Disabled,
    GeneralProject,
    F2LitMesh,
}

impl RuntimeProductProofPolicy {
    pub(in crate::entry) const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::GeneralProject => "general-project",
            Self::F2LitMesh => "f2-lit-mesh",
        }
    }
}
