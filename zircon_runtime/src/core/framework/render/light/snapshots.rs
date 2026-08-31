use crate::core::framework::render::RenderLayerSet;
use crate::core::framework::scene::EntityId;
use crate::core::framework::scene::Mobility;
use crate::core::math::{Real, Vec2, Vec3};

use super::shadow_settings::LightShadowSettings;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderDirectionalLightSnapshot {
    pub node_id: EntityId,
    pub light_id: u64,
    pub layer_mask: RenderLayerSet,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub mobility: Mobility,
    pub shadow: Option<LightShadowSettings>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderPointLightSnapshot {
    pub node_id: EntityId,
    pub light_id: u64,
    pub layer_mask: RenderLayerSet,
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub mobility: Mobility,
    pub shadow: Option<LightShadowSettings>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSpotLightSnapshot {
    pub node_id: EntityId,
    pub light_id: u64,
    pub layer_mask: RenderLayerSet,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub inner_angle_radians: Real,
    pub outer_angle_radians: Real,
    pub mobility: Mobility,
    pub shadow: Option<LightShadowSettings>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderAmbientLightSnapshot {
    pub color: Vec3,
    pub intensity: Real,
    /// Whether this source contributes ambient light to meshes with a baked lightmap.
    pub affects_lightmapped_meshes: bool,
    pub renderer_degraded: bool,
    pub degradation_reason: Option<String>,
}

impl Default for RenderAmbientLightSnapshot {
    fn default() -> Self {
        Self {
            color: Vec3::ZERO,
            intensity: 0.0,
            affects_lightmapped_meshes: true,
            renderer_degraded: true,
            degradation_reason: Some(
                "ambient light has no authored scene component yet".to_string(),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderRectLightSnapshot {
    pub node_id: EntityId,
    pub light_id: u64,
    pub layer_mask: RenderLayerSet,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub size: Vec2,
    pub shadow: Option<LightShadowSettings>,
    pub renderer_degraded: bool,
    pub degradation_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBakedLightingExtract {
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for RenderBakedLightingExtract {
    fn default() -> Self {
        Self {
            color: Vec3::ZERO,
            intensity: 0.0,
        }
    }
}
