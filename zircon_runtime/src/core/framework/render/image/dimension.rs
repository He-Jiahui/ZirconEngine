use serde::{Deserialize, Serialize};

/// Texture dimensionality for render-facing image descriptors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderImageDimension {
    D1,
    #[default]
    D2,
    D3,
}
