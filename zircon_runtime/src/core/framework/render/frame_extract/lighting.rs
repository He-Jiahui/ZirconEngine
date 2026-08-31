use super::super::{
    AdvancedLightingExtract, RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot,
    RenderHybridGiExtract, RenderPointLightSnapshot, RenderRectLightSnapshot,
    RenderSpotLightSnapshot,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LightingExtract {
    pub directional_lights: Vec<RenderDirectionalLightSnapshot>,
    pub point_lights: Vec<RenderPointLightSnapshot>,
    pub spot_lights: Vec<RenderSpotLightSnapshot>,
    pub ambient_lights: Vec<RenderAmbientLightSnapshot>,
    pub rect_lights: Vec<RenderRectLightSnapshot>,
    pub hybrid_global_illumination: Option<RenderHybridGiExtract>,
    pub advanced_lighting: AdvancedLightingExtract,
}
