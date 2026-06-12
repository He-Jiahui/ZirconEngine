use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialTextureTransform {
    #[serde(default = "default_texture_scale")]
    pub scale: [f32; 2],
    #[serde(default)]
    pub offset: [f32; 2],
}

impl RenderMaterialTextureTransform {
    pub const IDENTITY: Self = Self {
        scale: [1.0, 1.0],
        offset: [0.0, 0.0],
    };

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    pub fn as_uniform_vec4(self) -> [f32; 4] {
        [
            finite_or(self.scale[0], Self::IDENTITY.scale[0]),
            finite_or(self.scale[1], Self::IDENTITY.scale[1]),
            finite_or(self.offset[0], Self::IDENTITY.offset[0]),
            finite_or(self.offset[1], Self::IDENTITY.offset[1]),
        ]
    }
}

impl Default for RenderMaterialTextureTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn default_texture_scale() -> [f32; 2] {
    RenderMaterialTextureTransform::IDENTITY.scale
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}
