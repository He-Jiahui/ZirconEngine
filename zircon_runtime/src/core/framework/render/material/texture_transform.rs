use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialTextureTransform {
    #[serde(default = "default_texture_scale")]
    pub scale: [f32; 2],
    #[serde(default)]
    pub offset: [f32; 2],
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rotation: f32,
}

impl RenderMaterialTextureTransform {
    pub const IDENTITY: Self = Self {
        scale: [1.0, 1.0],
        offset: [0.0, 0.0],
        rotation: 0.0,
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

    /// Returns precomputed `(cos(rotation), sin(rotation))` for shader-side UV rotation.
    pub fn as_uniform_rotation_sin_cos(self) -> [f32; 2] {
        let (sin, cos) = finite_or(self.rotation, Self::IDENTITY.rotation).sin_cos();
        [cos, sin]
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

fn is_zero(value: &f32) -> bool {
    *value == 0.0
}

#[cfg(test)]
mod tests {
    use super::RenderMaterialTextureTransform;

    #[test]
    fn texture_transform_precomputes_rotation_sin_cos_with_finite_fallback() {
        assert_eq!(
            RenderMaterialTextureTransform::IDENTITY.as_uniform_rotation_sin_cos(),
            [1.0, 0.0]
        );

        let quarter_turn = RenderMaterialTextureTransform {
            rotation: std::f32::consts::FRAC_PI_2,
            ..RenderMaterialTextureTransform::IDENTITY
        };
        let [cos, sin] = quarter_turn.as_uniform_rotation_sin_cos();
        assert!(cos.abs() <= 0.000_001);
        assert!((sin - 1.0).abs() <= 0.000_001);

        let non_finite = RenderMaterialTextureTransform {
            rotation: f32::NAN,
            ..RenderMaterialTextureTransform::IDENTITY
        };
        assert_eq!(non_finite.as_uniform_rotation_sin_cos(), [1.0, 0.0]);
    }
}
