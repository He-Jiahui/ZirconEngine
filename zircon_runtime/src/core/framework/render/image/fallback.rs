use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderImageFallbackKind {
    #[default]
    MissingImage,
    OpaqueWhite,
    TransparentBlack,
    NormalMap,
}
