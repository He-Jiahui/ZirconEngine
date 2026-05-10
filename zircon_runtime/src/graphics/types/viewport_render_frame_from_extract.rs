use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub fn from_extract(extract: RenderFrameExtract, viewport_size: impl Into<UVec2>) -> Self {
        let viewport_size = viewport_size.into();
        let scene = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: crate::core::math::Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        Self {
            scene,
            extract,
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            ui: None,
            virtual_geometry_debug_snapshot: None,
            prepared_runtime_sidebands: Default::default(),
        }
    }
}
