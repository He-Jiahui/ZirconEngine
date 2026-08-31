use crate::core::framework::scene::EntityId;
use crate::core::math::UVec2;

use super::super::camera_stack::CameraRenderDescriptor;
use super::super::RenderVirtualGeometryDebugState;
use super::ViewportRenderSettings;

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportExtractRequest {
    pub settings: ViewportRenderSettings,
    pub active_camera_override: Option<EntityId>,
    pub camera: Option<CameraRenderDescriptor>,
    pub viewport_size: Option<UVec2>,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}

impl Default for SceneViewportExtractRequest {
    fn default() -> Self {
        Self {
            settings: ViewportRenderSettings::default(),
            active_camera_override: None,
            camera: None,
            viewport_size: None,
            virtual_geometry_debug: None,
        }
    }
}
