use super::*;

const SKINNED_MORPH_VELOCITY_PNG_STATUS: &str =
    "render_plan08_skinned_morph_weight_velocity_product_png_passed_renderdoc_deferred";

#[test]
fn render_product_skinned_mesh_morph_weight_change_writes_scene_velocity_pixels() {
    let _capture = capture_skinned_morph_velocity_product();
}

#[test]
#[ignore = "writes Plan 08 product scene-velocity PNG artifact"]
fn export_skinned_morph_weight_velocity_product_png() {
    let capture = capture_skinned_morph_velocity_product();
    let output_path = velocity_png::render_test_output_dir()
        .join("runtime_render_plan08_skinned_morph_weight_velocity_20260703.png");
    velocity_png::save_scene_velocity_png(
        &capture.framework,
        capture.viewport_size,
        &output_path,
        SKINNED_MORPH_VELOCITY_PNG_STATUS,
    );
}

struct SkinnedMorphVelocityCapture {
    framework: WgpuRenderFramework,
    viewport_size: UVec2,
}

fn capture_skinned_morph_velocity_product() -> SkinnedMorphVelocityCapture {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri =
        AssetUri::parse("res://materials/product-skinned-morph-velocity.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    register_unlit_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "product-skinned-morph-velocity-material-v1",
    );
    let mesh_uri = AssetUri::parse("res://meshes/product-skinned-morph-velocity.zmesh").unwrap();
    let mesh_id = ResourceId::from_locator(&mesh_uri);
    register_skinned_morph_mesh_revision(
        &asset_manager,
        mesh_id,
        mesh_uri,
        "product-skinned-morph-velocity-mesh-v1",
    );
    let skeleton_uri =
        AssetUri::parse("res://animation/product-skinned-morph-velocity.skeleton.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    register_skinned_morph_skeleton_revision(
        &asset_manager,
        skeleton_id,
        skeleton_uri,
        "product-skinned-morph-velocity-skeleton-v1",
    );

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(128, 128);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, morph_velocity_quality_profile())
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            skinned_morph_velocity_extract(material_id, mesh_id, skeleton_id, 2701, 0.0),
        )
        .unwrap();
    let first = framework.query_stats().unwrap();
    assert_eq!(
        first.last_mesh_gpu_skinned_morphed_source_draw_count, 0,
        "the first all-zero skinned morph frame should establish previous morph state without drawing a skinned morph payload"
    );
    assert_eq!(
        first.last_mesh_skinned_gpu_skinning_draw_count, 1,
        "the first frame should keep skinned meshes on the shader-skinning path"
    );

    framework
        .submit_frame_extract(
            viewport,
            skinned_morph_velocity_extract(material_id, mesh_id, skeleton_id, 2702, 1.0),
        )
        .unwrap();
    let second = framework.query_stats().unwrap();

    assert_eq!(second.last_mesh_gpu_skinned_morphed_source_draw_count, 1);
    assert_eq!(second.last_mesh_skinned_gpu_skinning_draw_count, 1);
    assert_eq!(second.last_mesh_previous_velocity_transform_draw_count, 1);
    assert_eq!(second.last_mesh_missing_velocity_transform_draw_count, 0);
    assert_scene_velocity_readback_nonzero(
        &second,
        viewport_size,
        "skinned morph weight 0 -> 1 should write object velocity from previous morph weights",
    );
    SkinnedMorphVelocityCapture {
        framework,
        viewport_size,
    }
}

fn skinned_morph_velocity_extract(
    material_id: ResourceId,
    mesh_id: ResourceId,
    skeleton_id: ResourceId,
    world: u64,
    morph_weight: f32,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Perspective,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(UVec2::new(128, 128));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(camera.projection_mode),
    );
    let mut descriptor =
        CameraRenderDescriptor::from_camera_payload(Some(SKINNED_MORPH_VELOCITY_NODE_ID), camera);
    descriptor.clear = RenderCameraClear::Color(Vec4::ZERO);
    extract.view.select_camera_descriptor(descriptor);
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![skinned_morph_velocity_mesh_snapshot(
            material_id,
            mesh_id,
            morph_weight,
        )],
    );
    extract.animation_poses = vec![RenderSkeletalPoseExtract {
        entity: SKINNED_MORPH_VELOCITY_NODE_ID,
        skeleton: skeleton_id,
        pose: skinned_morph_pose(),
    }];
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract.post_process.effect_stack.motion_blur = RenderMotionBlurSettings {
        shutter_angle: 0.5,
        samples: 1,
    };
    extract
}

fn skinned_morph_velocity_mesh_snapshot(
    material_id: ResourceId,
    mesh_id: ResourceId,
    morph_weight: f32,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: SKINNED_MORPH_VELOCITY_NODE_ID,
        stable_instance_key: SKINNED_MORPH_VELOCITY_NODE_ID << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: Some(ResourceHandle::<MeshMarker>::new(mesh_id)),
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: vec![morph_weight],
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            ..Default::default()
        },
    }
}

const SKINNED_MORPH_VELOCITY_NODE_ID: u64 = 2701;
