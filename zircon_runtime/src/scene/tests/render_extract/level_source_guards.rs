use super::*;

use std::sync::Arc;

#[test]
fn world_render_extract_stamps_the_exact_source_world_generation() {
    let mut world = World::empty();
    world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");

    let extract = world.to_render_frame_extract();

    assert_eq!(extract.world.raw(), 0);
    assert_eq!(extract.world.generation(), world.world_generation());
    assert_ne!(extract.world.generation(), 0);
}

#[test]
#[cfg(feature = "animation")]
fn level_system_render_extract_uses_world_direct_path_and_merges_animation_poses() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), Default::default());
    let (mesh_with_skeleton, mesh_without_skeleton, skeleton_handle) =
        level.with_world_mut(|world| {
            let camera = spawn_camera_on_layer(world, 0b1111);
            world.set_active_camera(camera);
            let mesh_with_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
            let mesh_without_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
            let skeleton_handle = ResourceHandle::<AnimationSkeletonMarker>::new(
                ResourceId::from_stable_label("res://animation/hero.skeleton.zranim"),
            );
            world
                .set_animation_skeleton(
                    mesh_with_skeleton,
                    Some(AnimationSkeletonComponent {
                        skeleton: skeleton_handle,
                    }),
                )
                .unwrap();
            (mesh_with_skeleton, mesh_without_skeleton, skeleton_handle)
        });
    let missing_entity = 99_999;
    let pose = test_pose("hip");
    let replacement_epoch = level.capture_world_replacement_epoch();
    assert!(level.record_animation_pose_snapshot(
        replacement_epoch,
        Arc::new(BTreeMap::from([
            (mesh_with_skeleton, Arc::new(pose.clone())),
            (
                mesh_without_skeleton,
                Arc::new(test_pose("filtered-no-skeleton")),
            ),
            (missing_entity, Arc::new(test_pose("filtered-missing"))),
        ]))
    ));

    let extract = RenderExtractProducer::build_render_frame_extract(
        &level,
        &RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(705),
            SceneViewportExtractRequest::default(),
        ),
    );

    assert_eq!(extract.world.raw(), 705);
    assert!(
        extract
            .geometry
            .meshes
            .iter()
            .any(|mesh| mesh.node_id == mesh_with_skeleton)
    );
    assert_eq!(extract.animation_poses.len(), 1);
    assert_eq!(extract.animation_poses[0].entity, mesh_with_skeleton);
    assert_eq!(extract.animation_poses[0].skeleton, skeleton_handle.id());
    assert_eq!(extract.animation_poses[0].pose.as_ref(), &pose);
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));
}

#[test]
#[cfg(feature = "animation")]
fn level_frame_snapshot_publishes_a_new_animation_generation_without_retiring_the_old_handle() {
    let level = DefaultLevelManager::default().create_default_level();
    let entity = level.with_world_mut(|world| {
        world
            .spawn_node(NodeKind::Mesh)
            .expect("test scene spawn should succeed")
    });
    let pose = test_pose("frame-snapshot");
    let initial = level.frame_state_snapshot();
    let replacement_epoch = level.capture_world_replacement_epoch();

    assert!(level.record_animation_pose_snapshot(
        replacement_epoch,
        Arc::new(BTreeMap::from([(entity, Arc::new(pose.clone()))])),
    ));
    let published = level.frame_state_snapshot();

    assert_eq!(initial.animation_generation(), 0);
    assert!(initial.animation_poses().is_empty());
    assert_eq!(published.animation_generation(), 1);
    assert_eq!(
        published.animation_poses().get(&entity).map(Arc::as_ref),
        Some(&pose)
    );
    assert!(
        !std::sync::Arc::ptr_eq(initial.animation_poses(), published.animation_poses()),
        "a new animation publication must not mutate an earlier frame handle"
    );

    assert!(level.record_animation_pose_snapshot(
        replacement_epoch,
        Arc::new(BTreeMap::from([(entity, Arc::new(pose.clone()))])),
    ));
    let unchanged = level.frame_state_snapshot();
    assert_eq!(
        unchanged.animation_generation(),
        published.animation_generation()
    );
    assert!(
        std::sync::Arc::ptr_eq(unchanged.animation_poses(), published.animation_poses()),
        "an unchanged pose payload must retain its sealed frame handle"
    );

    assert!(level.record_animation_pose_snapshot(replacement_epoch, Arc::default()));
    let cleared = level.frame_state_snapshot();
    assert_eq!(cleared.animation_generation(), 2);
    assert!(cleared.animation_poses().is_empty());
    assert_eq!(
        published.animation_poses().get(&entity).map(Arc::as_ref),
        Some(&pose)
    );

    let published_world_generation = published.world_generation();
    level.replace_world_and_reset_runtime_state(World::empty());
    let replaced = level.frame_state_snapshot();
    assert_ne!(replaced.world_generation(), published_world_generation);
    assert!(replaced.animation_poses().is_empty());
    assert_eq!(
        published.animation_poses().get(&entity).map(Arc::as_ref),
        Some(&pose)
    );
}

#[test]
#[cfg(feature = "animation")]
fn level_system_render_extract_consumes_the_sealed_animation_frame_handle_without_resorting() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("level_system_render_extract.rs"),
    )
    .unwrap();

    assert!(
        source.contains("let frame_state = self.frame_state_snapshot();")
            && source.contains("frame_state.animation_poses()")
            && source.contains("frame_state.world_generation() != world.world_generation()")
            && source.contains(".iter()")
            && !source.contains("animation_pose_entries"),
        "scene render extraction must consume the sealed Arc-backed BTreeMap handle instead of cloning poses before filtering"
    );
    assert!(
        !source.contains("animation_poses.sort") && !source.contains("sort_by_key"),
        "the animation-pose cache is a BTreeMap, so filtering its ordered iterator must not add an O(n log n) per-frame resort"
    );
}

#[test]
fn render_frame_extract_snapshot_adapters_are_not_scene_production_paths() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "src/scene/render_extract/mod.rs",
        "src/scene/world/render.rs",
        "src/scene/level_system_render_extract.rs",
    ] {
        assert_source_excludes_file(
            &manifest_root.join(relative),
            &["RenderFrameExtract::from_snapshot"],
            "scene production extraction must populate RenderFrameExtract directly; snapshot adapters are allowed only for preview/test/roundtrip/synthetic helpers",
        );
    }

    let submit_root = manifest_root
        .join("src")
        .join("graphics")
        .join("runtime")
        .join("render_framework")
        .join("submit_frame_extract");
    assert_runtime_submit_tree_excludes_snapshot_adapters(&submit_root);
}

#[test]
fn render_view_extract_keeps_selected_scene_camera_descriptor_when_inactive() {
    let render_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("render.rs"),
    )
    .unwrap();
    let view_builder = render_source
        .split("fn build_render_view_extract")
        .nth(1)
        .and_then(|text| text.split("fn render_extract_layers_for_view").next())
        .expect("read render view extract builder");

    assert!(
        view_builder.contains("descriptor.entity == Some(entity) || descriptor.is_active()")
            && !view_builder.contains(".filter(CameraRenderDescriptor::is_active)"),
        "scene render view extraction must keep the selected camera descriptor even when the camera is inactive"
    );
}
