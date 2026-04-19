use zircon_framework::render::{
    HandleOverlayExtract, SceneViewportSettings, ViewportCameraSnapshot,
};
use zircon_math::UVec2;
use zircon_scene::Scene;

use crate::scene::viewport::pointer::{
    candidates::{renderable_candidates, scene_gizmo_candidates},
    viewport_pointer_layout::ViewportPointerLayout,
};

use super::ViewportOverlayPointerBridge;

impl ViewportOverlayPointerBridge {
    pub(crate) fn sync(&mut self, layout: ViewportPointerLayout) {
        self.layout = layout;
        self.rebuild_surface();
    }

    pub(crate) fn sync_scene(
        &mut self,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
        handles: Vec<HandleOverlayExtract>,
    ) {
        self.sync(ViewportPointerLayout {
            viewport,
            camera: camera.clone(),
            handles,
            scene_gizmos: scene_gizmo_candidates(scene, selected, settings, camera),
            renderables: renderable_candidates(scene),
        });
    }
}
