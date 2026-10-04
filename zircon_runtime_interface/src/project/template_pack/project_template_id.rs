use serde::{Deserialize, Serialize};

/// Stable identifier for a packaged project template.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectTemplateId {
    #[default]
    RenderableEmpty,
}

impl ProjectTemplateId {
    /// 产生写入清单、Hub 请求和 descriptor 资格名时使用的唯一小写 ID。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RenderableEmpty => "renderable-empty",
        }
    }

    /// Parses one canonical built-in template ID without compatibility normalization.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "renderable-empty" => Some(Self::RenderableEmpty),
            _ => None,
        }
    }

    /// Returns the immutable package descriptor used by all project creation surfaces.
    pub fn descriptor(self) -> super::ProjectTemplateDescriptor {
        super::project_template_descriptor(self)
    }
}

#[cfg(test)]
#[path = "tests/project_template_id.rs"]
mod tests;
