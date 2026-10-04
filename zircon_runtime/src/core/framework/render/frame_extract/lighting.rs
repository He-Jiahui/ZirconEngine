use super::super::{
    AdvancedLightingExtract, RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot,
    RenderHybridGiExtract, RenderPointLightSnapshot, RenderRectLightSnapshot,
    RenderSpotLightSnapshot,
};

/// 场景提取时按视图层筛选的光照输入，供后续阴影和高级光照准备消费。
/// 多相机提交共享该场景域，生产端必须覆盖所有将被提交相机的光源需求。
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
