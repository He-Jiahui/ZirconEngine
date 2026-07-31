use std::sync::Arc;

use crate::scene::viewport::{ViewportCameraSnapshot, ViewportInteractionExtract};
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::pointer::{
    candidates::renderable_candidates, viewport_pointer_layout::ViewportPointerLayout,
};

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn sync(&mut self, layout: ViewportPointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        self.layout = layout;
        self.interaction_extract = None;
        self.rebuild_surface();
        true
    }

    pub(crate) fn sync_scene(
        &mut self,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
        interaction_extract: Arc<ViewportInteractionExtract>,
    ) -> bool {
        if self
            .interaction_extract
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &interaction_extract))
        {
            return false;
        }

        let changed = self.sync(ViewportPointerLayout {
            viewport,
            camera: camera.clone(),
            handles: interaction_extract.handles(),
            scene_gizmos: interaction_extract.scene_gizmos(),
            renderables: renderable_candidates(interaction_extract.render_meshes()).into(),
        });
        self.interaction_extract = Some(interaction_extract);
        changed
    }
}
