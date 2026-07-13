use serde::{Deserialize, Serialize};

/// Stable identifier for a packaged project template.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectTemplateId {
    #[default]
    RenderableEmpty,
}

impl ProjectTemplateId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RenderableEmpty => "renderable-empty",
        }
    }
}
