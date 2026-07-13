use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::ProjectTemplateId;

/// Editor-facing template choice mapped to the shared packaged-template identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewProjectTemplate {
    #[default]
    RenderableEmpty,
}

impl NewProjectTemplate {
    pub const fn pack_id(self) -> ProjectTemplateId {
        match self {
            Self::RenderableEmpty => ProjectTemplateId::RenderableEmpty,
        }
    }
}
