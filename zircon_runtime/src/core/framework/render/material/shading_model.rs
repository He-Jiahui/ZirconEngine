use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use super::RenderMaterialLightingModel;

pub const SHADING_MODEL_ID_UNLIT: ShadingModelId = ShadingModelId::new(0);
pub const SHADING_MODEL_ID_BLINN_PHONG: ShadingModelId = ShadingModelId::new(1);
pub const SHADING_MODEL_ID_STANDARD_PBR: ShadingModelId = ShadingModelId::new(2);
pub const SHADING_MODEL_PLUGIN_ID_START: u8 = 16;
pub const SHADING_MODEL_GBUFFER_ALPHA_SCALE: f32 = 255.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShadingModelId(u8);

impl ShadingModelId {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub const fn is_plugin_range(self) -> bool {
        self.0 >= SHADING_MODEL_PLUGIN_ID_START
    }

    pub fn encode_gbuffer_alpha(self) -> f32 {
        f32::from(self.0) / SHADING_MODEL_GBUFFER_ALPHA_SCALE
    }

    pub fn decode_gbuffer_alpha(value: f32) -> Self {
        if !value.is_finite() {
            return SHADING_MODEL_ID_STANDARD_PBR;
        }
        let clamped = value.clamp(0.0, 1.0);
        Self::new((clamped * SHADING_MODEL_GBUFFER_ALPHA_SCALE).round() as u8)
    }

    pub fn from_lighting_model(model: &RenderMaterialLightingModel) -> Option<Self> {
        match model {
            RenderMaterialLightingModel::Pbr => Some(SHADING_MODEL_ID_STANDARD_PBR),
            RenderMaterialLightingModel::BlinnPhong => Some(SHADING_MODEL_ID_BLINN_PHONG),
            RenderMaterialLightingModel::Unlit => Some(SHADING_MODEL_ID_UNLIT),
            RenderMaterialLightingModel::Custom { .. } => None,
        }
    }
}

impl Display for ShadingModelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GBufferChannelMask(u16);

impl GBufferChannelMask {
    pub const EMPTY: Self = Self(0);
    pub const ALBEDO: Self = Self(1 << 0);
    pub const NORMAL: Self = Self(1 << 1);
    pub const MATERIAL: Self = Self(1 << 2);
    pub const DEPTH: Self = Self(1 << 3);
    pub const CUSTOM0: Self = Self(1 << 8);
    pub const CUSTOM1: Self = Self(1 << 9);

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, required: Self) -> bool {
        (self.0 & required.0) == required.0
    }

    pub const fn standard_deferred_v1() -> Self {
        Self::ALBEDO
            .union(Self::NORMAL)
            .union(Self::MATERIAL)
            .union(Self::DEPTH)
    }

    pub const fn standard_lit() -> Self {
        Self::ALBEDO.union(Self::NORMAL).union(Self::MATERIAL)
    }

    pub const fn unlit() -> Self {
        Self::ALBEDO
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadingModelDescriptor {
    pub id: ShadingModelId,
    pub token: String,
    pub forward_include: String,
    pub gbuffer_encode_include: String,
    pub deferred_include: String,
    pub required_channels: GBufferChannelMask,
}

impl ShadingModelDescriptor {
    pub fn new(
        id: ShadingModelId,
        token: impl Into<String>,
        forward_include: impl Into<String>,
        gbuffer_encode_include: impl Into<String>,
        deferred_include: impl Into<String>,
        required_channels: GBufferChannelMask,
    ) -> Self {
        Self {
            id,
            token: token.into(),
            forward_include: forward_include.into(),
            gbuffer_encode_include: gbuffer_encode_include.into(),
            deferred_include: deferred_include.into(),
            required_channels,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShadingModelRegistrationError {
    DuplicateId {
        id: ShadingModelId,
        existing_token: String,
        new_token: String,
    },
    DuplicateToken {
        token: String,
        existing_id: ShadingModelId,
        new_id: ShadingModelId,
    },
    RequiredChannelsUnsupported {
        token: String,
        required: GBufferChannelMask,
        supported: GBufferChannelMask,
    },
    PluginIdReserved {
        token: String,
        id: ShadingModelId,
        minimum: u8,
    },
}

impl Display for ShadingModelRegistrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId {
                id,
                existing_token,
                new_token,
            } => write!(
                f,
                "shading model id {id} is already registered by {existing_token} and cannot be reused by {new_token}"
            ),
            Self::DuplicateToken {
                token,
                existing_id,
                new_id,
            } => write!(
                f,
                "shading model token {token} is already registered as id {existing_id} and cannot be reused by id {new_id}"
            ),
            Self::RequiredChannelsUnsupported {
                token,
                required,
                supported,
            } => write!(
                f,
                "shading model {token} requires G-buffer channels {:#x}, but the current layout supports {:#x}",
                required.bits(),
                supported.bits()
            ),
            Self::PluginIdReserved { token, id, minimum } => write!(
                f,
                "plugin shading model {token} uses id {id}, but plugin shading model ids must be >= {minimum}"
            ),
        }
    }
}

impl std::error::Error for ShadingModelRegistrationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_material_lighting_model_token_resolves_shading_id() {
        assert_eq!(
            ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::Pbr),
            Some(SHADING_MODEL_ID_STANDARD_PBR)
        );
        assert_eq!(
            ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::BlinnPhong),
            Some(SHADING_MODEL_ID_BLINN_PHONG)
        );
        assert_eq!(
            ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::Unlit),
            Some(SHADING_MODEL_ID_UNLIT)
        );
        assert_eq!(
            ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::Custom {
                name: "subsurface".to_string()
            }),
            None
        );
    }

    #[test]
    fn render_material_shading_model_id_roundtrips_gbuffer_encoding() {
        for id in [
            SHADING_MODEL_ID_UNLIT,
            SHADING_MODEL_ID_BLINN_PHONG,
            SHADING_MODEL_ID_STANDARD_PBR,
            ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START),
            ShadingModelId::new(u8::MAX),
        ] {
            assert_eq!(
                ShadingModelId::decode_gbuffer_alpha(id.encode_gbuffer_alpha()),
                id
            );
        }
    }

    #[test]
    fn render_material_custom_lighting_model_waits_for_plugin_registration() {
        assert!(ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START).is_plugin_range());
        assert!(!SHADING_MODEL_ID_STANDARD_PBR.is_plugin_range());
    }

    #[test]
    fn gbuffer_channel_mask_reports_required_channel_overflow() {
        let supported = GBufferChannelMask::standard_deferred_v1();
        assert!(supported.contains(GBufferChannelMask::standard_lit()));
        assert!(!supported.contains(GBufferChannelMask::CUSTOM0));
    }
}
