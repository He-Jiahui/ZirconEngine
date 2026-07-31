use crate::scene::viewport::{SceneViewportSettings, ViewportCameraSnapshot};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ViewportInteractionExtractKey {
    world_generation: u64,
    selected: Option<u64>,
    settings: SceneViewportSettings,
    camera: ViewportCameraSnapshot,
    viewport: UVec2,
}

impl ViewportInteractionExtractKey {
    pub(super) fn new(
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> Self {
        Self {
            world_generation: scene.world_generation(),
            selected,
            settings: settings.clone(),
            camera: camera.clone(),
            viewport,
        }
    }
}
