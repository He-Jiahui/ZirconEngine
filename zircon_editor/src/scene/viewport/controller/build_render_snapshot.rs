use crate::scene::viewport::{
    RenderFrameExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
};
use zircon_runtime::scene::Scene;

use super::SceneViewportController;
use crate::scene::viewport::render_packet::{build_grid_extract, build_selection_anchors};

pub(super) struct CachedRenderScene {
    generation: u64,
    request: SceneViewportExtractRequest,
    extract: RenderFrameExtract,
}

impl SceneViewportController {
    /// Compatibility fixture for tests holding an immutable scene. Production uses the
    /// mutable authoring-world gateway and never clones the World.
    #[cfg(test)]
    pub(crate) fn build_render_snapshot(&self, scene: &Scene) -> RenderSceneSnapshot {
        self.build_prepared_render_extract(&mut scene.clone())
            .to_scene_snapshot()
    }

    pub(crate) fn build_prepared_render_extract(&self, scene: &mut Scene) -> RenderFrameExtract {
        scene.prepare_viewport_render_extract();
        let camera = self.current_camera(scene);
        let selected = self.projected_selected_node(scene);
        let request = SceneViewportExtractRequest {
            settings: self.state.settings.render_settings(),
            active_camera_override: None,
            camera: Some(camera.clone().into()),
            viewport_size: Some(self.state.viewport.size),
            virtual_geometry_debug: None,
        };
        let generation = scene.world_generation();
        let mut cache = self.render_cache.borrow_mut();
        if !cache
            .as_ref()
            .is_some_and(|cached| cached.generation == generation && cached.request == request)
        {
            let packet = scene.build_prepared_viewport_render_packet(&request);
            *cache = Some(CachedRenderScene {
                generation: scene.world_generation(),
                request,
                extract: RenderFrameExtract::from_snapshot(
                    RenderWorldSnapshotHandle::new(scene.world_generation()),
                    packet,
                ),
            });
        }
        let mut extract = cache
            .as_ref()
            .expect("render cache populated")
            .extract
            .clone();
        drop(cache);
        let interaction = self.interaction_extract.resolve_from_render_packet(
            scene,
            selected,
            &self.state.settings,
            &camera,
            self.state.viewport.size,
            &extract.geometry.meshes,
            || self.handle_overlays(scene, &camera),
            || self.viewport_overlay_gizmos(scene, selected),
        );
        // Only the small overlay domain is made unique; geometry and lighting stay shared.
        let overlays = &mut extract.debug.overlays;
        overlays.selection_anchors = build_selection_anchors(scene, selected, &self.state.settings);
        overlays.grid = build_grid_extract(&self.state.settings);
        overlays.handles = interaction.handles().as_ref().to_vec();
        overlays.scene_gizmos = interaction.scene_gizmos().as_ref().to_vec();
        self.apply_runtime_highlights(overlays);
        extract
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::math::UVec2;

    #[test]
    fn viewport_cache_reuses_geometry_and_invalidates_for_world_edits_and_reset() {
        let mut scene = Scene::new();
        let mut controller = SceneViewportController::new(UVec2::new(800, 600));
        let first = controller.build_prepared_render_extract(&mut scene);
        let second = controller.build_prepared_render_extract(&mut scene);
        assert!(first.geometry.ptr_eq(&second.geometry));
        assert!(first.lighting.ptr_eq(&second.lighting));
        scene
            .spawn_node(zircon_runtime::scene::NodeKind::Cube)
            .unwrap();
        let edited = controller.build_prepared_render_extract(&mut scene);
        assert!(!second.geometry.ptr_eq(&edited.geometry));
        controller.reset_from_scene(Some(&scene));
        let reset = controller.build_prepared_render_extract(&mut scene);
        assert!(!edited.geometry.ptr_eq(&reset.geometry));
    }
}
