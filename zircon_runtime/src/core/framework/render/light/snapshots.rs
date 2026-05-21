use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec2, Vec3};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderDirectionalLightSnapshot {
    pub node_id: EntityId,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderPointLightSnapshot {
    pub node_id: EntityId,
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSpotLightSnapshot {
    pub node_id: EntityId,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub inner_angle_radians: Real,
    pub outer_angle_radians: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderAmbientLightSnapshot {
    pub color: Vec3,
    pub intensity: Real,
    pub renderer_degraded: bool,
    pub degradation_reason: Option<String>,
}

impl Default for RenderAmbientLightSnapshot {
    fn default() -> Self {
        Self {
            color: Vec3::ZERO,
            intensity: 0.0,
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
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub size: Vec2,
    pub renderer_degraded: bool,
    pub degradation_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderReflectionProbeSnapshot {
    pub position: Vec3,
    pub radius: Real,
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for RenderReflectionProbeSnapshot {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            radius: 0.0,
            color: Vec3::ZERO,
            intensity: 0.0,
        }
    }
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
