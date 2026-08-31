use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Vec3;

use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;

use super::{viewport_hover_state::ViewportHoverState, SceneViewportController};

impl SceneViewportController {
    pub(crate) fn reset_from_scene(&mut self, scene: Option<&Scene>) {
        self.interaction_extract.invalidate();
        self.reset_camera_from_scene(scene);
        self.state.orbit_target = scene
            .and_then(|scene| {
                Self::selected_world_position(scene, self.state.selection.active_primary())
            })
            .unwrap_or(Vec3::ZERO);
        self.state
            .orbit_controller
            .set_target(self.state.orbit_target);
        self.state.hover = ViewportHoverState::default();
        self.state.drag = None;
        self.pointer_bridge = ViewportOverlayPointerRouter::new();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime::scene::Scene;
    use zircon_runtime_interface::math::UVec2;

    use crate::scene::viewport::ViewportCameraSnapshot;

    use super::SceneViewportController;

    #[test]
    fn resetting_for_a_replacement_world_invalidates_the_shared_interaction_extract() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let previous_world = Scene::new();
        let replacement_world = Scene::new();
        let settings = controller.settings().clone();
        let camera = ViewportCameraSnapshot::default();
        let viewport = UVec2::new(1280, 720);

        assert_eq!(
            previous_world.world_generation(),
            replacement_world.world_generation(),
            "the regression requires equal generations from distinct world owners"
        );
        controller.build_render_snapshot(&previous_world);
        let before = controller.interaction_extract.resolve_for_pointer(
            &previous_world,
            None,
            &settings,
            &camera,
            viewport,
        );
        let crate::scene::viewport::ViewportInteractionExtractPointerResolution::Ready(before) =
            before
        else {
            panic!("the render path must publish a pointer extract before reset");
        };

        controller.reset_from_scene(Some(&replacement_world));

        assert!(matches!(
            controller.interaction_extract.resolve_for_pointer(
                &replacement_world,
                None,
                &settings,
                &camera,
                viewport,
            ),
            crate::scene::viewport::ViewportInteractionExtractPointerResolution::Stale
        ));
        controller.build_render_snapshot(&replacement_world);
        let after = controller.interaction_extract.resolve_for_pointer(
            &replacement_world,
            None,
            &settings,
            &camera,
            viewport,
        );
        let crate::scene::viewport::ViewportInteractionExtractPointerResolution::Ready(after) =
            after
        else {
            panic!("the replacement render path must publish a new pointer extract");
        };
        assert!(!Arc::ptr_eq(&before, &after));
    }
}
