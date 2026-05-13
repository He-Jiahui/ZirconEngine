use crate::scene::viewport::{HandleOverlayExtract, SceneViewportSettings, ViewportCameraSnapshot};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::pointer::{
    candidates::{renderable_candidates, scene_gizmo_candidates},
    viewport_pointer_layout::ViewportPointerLayout,
};

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn sync(&mut self, layout: ViewportPointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        self.layout = layout;
        self.rebuild_surface();
        true
    }

    pub(crate) fn sync_scene(
        &mut self,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
        handles: Vec<HandleOverlayExtract>,
    ) -> bool {
        self.sync(ViewportPointerLayout {
            viewport,
            camera: camera.clone(),
            handles,
            scene_gizmos: scene_gizmo_candidates(scene, selected, settings, camera),
            renderables: renderable_candidates(scene),
        })
    }
}
