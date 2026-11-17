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
    use zircon_runtime::scene::NodeKind;
    use zircon_runtime_interface::math::{Quat, Transform, UVec2, Vec3};

    // Same input as Three.js r180 Object3D.tests.js `localToWorld`.
    #[test]
    fn hierarchy_local_to_world_matches_three_js_parent_chain() {
        let mut world = Scene::empty();
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        let child = world.spawn_node(NodeKind::Empty).unwrap();
        world.set_parent_checked(child, Some(parent)).unwrap();
        world
            .update_transform(
                parent,
                Transform::from_translation(Vec3::X)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(2.0, 1.0, 1.0)),
            )
            .unwrap();
        world
            .update_transform(
                child,
                Transform::from_translation(Vec3::Y)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(1.0, 2.0, 1.0)),
            )
            .unwrap();
        let point = Vec3::splat(2.0);
        let expected = Vec3::new(5.0, -1.0, -4.0);
        // Check both the on-demand parent chain and the published render state.
        let before = world.world_matrix(child).unwrap().transform_point3(point);
        world.prepare_viewport_render_extract();
        let after = world.world_matrix(child).unwrap().transform_point3(point);
        assert!((before - expected).length() < 1.0e-5, "{before:?}");
        assert!((after - expected).length() < 1.0e-5, "{after:?}");
    }

    #[test]
    fn viewport_mesh_revision_distinguishes_opposite_shear_with_equal_trs() {
        let mut scene = Scene::new();
        let cube = scene
            .nodes()
            .iter()
            .find(|node| node.kind == NodeKind::Cube)
            .unwrap()
            .id;
        let parent = scene.spawn_node(NodeKind::Empty).unwrap();
        scene.set_parent_checked(cube, Some(parent)).unwrap();
        let controller = SceneViewportController::new(UVec2::splat(256));

        // Same opposite-shear witness as the runtime prepared-draw regression.
        let z = f32::from_bits(0x3ec3ef15);
        let w = f32::from_bits(0x3f6c8360);
        let mut extract = |sign: f32| {
            scene
                .update_transform(
                    parent,
                    Transform::default()
                        .with_rotation(Quat::from_xyzw(0.0, 0.0, z * sign, w))
                        .with_scale(Vec3::new(2.0, 1.0, 1.0)),
                )
                .unwrap();
            scene
                .update_transform(
                    cube,
                    Transform::from_translation(Vec3::new(0.0, 0.0, -5.0))
                        .with_rotation(Quat::from_xyzw(0.0, 0.0, -z * sign, w)),
                )
                .unwrap();
            let frame = controller.build_prepared_render_extract(&mut scene);
            let current = controller.build_prepared_render_extract(&mut scene);
            assert!(current.geometry.ptr_eq(&frame.geometry));
            let mesh = frame
                .geometry
                .meshes
                .iter()
                .find(|mesh| mesh.node_id == cube)
                .unwrap();
            (
                scene.world_matrix(cube).unwrap(),
                mesh.transform,
                mesh.stable_instance_key,
                mesh.transform_revision,
            )
        };
        let (plus, plus_trs, key, revision) = extract(1.0);
        let (minus, minus_trs, next_key, next_revision) = extract(-1.0);
        assert!(plus.x_axis.y > 0.45 && minus.x_axis.y < -0.45);
        assert_eq!(plus_trs, minus_trs, "fixture must hide the shear in TRS");
        assert_eq!(key, next_key);
        assert_ne!(
            revision, next_revision,
            "editor frame lost its inherited shear revision"
        );
    }

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
