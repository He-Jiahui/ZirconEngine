use super::*;

#[test]
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
    level.record_animation_poses(BTreeMap::from([
        (mesh_with_skeleton, pose.clone()),
        (mesh_without_skeleton, test_pose("filtered-no-skeleton")),
        (missing_entity, test_pose("filtered-missing")),
    ]));

    let extract = RenderExtractProducer::build_render_frame_extract(
        &level,
        &RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(705),
            SceneViewportExtractRequest::default(),
        ),
    );

    assert_eq!(extract.world.raw(), 705);
    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == mesh_with_skeleton));
    assert_eq!(extract.animation_poses.len(), 1);
    assert_eq!(extract.animation_poses[0].entity, mesh_with_skeleton);
    assert_eq!(extract.animation_poses[0].skeleton, skeleton_handle.id());
    assert_eq!(extract.animation_poses[0].pose, pose);
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));
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
