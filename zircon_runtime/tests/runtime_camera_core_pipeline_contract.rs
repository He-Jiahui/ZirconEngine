use zircon_runtime::asset::SceneCameraAsset;
use zircon_runtime::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderExtractContext, RenderExtractProducer,
    RenderWorldSnapshotHandle, SceneViewportExtractRequest, ViewProjectionMatrixPair,
    ViewportCameraSnapshot,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::scene::components::CameraComponent;
use zircon_runtime::scene::{NodeKind, World};

#[test]
fn orthographic_projection_and_core_pipeline_are_independent_public_contracts() {
    let core3d = orthographic_extract(CorePipelineKind::Core3d);
    let core2d = orthographic_extract(CorePipelineKind::Core2d);

    assert_eq!(
        core3d.view.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(
        core2d.view.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(core3d.view.core_pipeline, CorePipelineKind::Core3d);
    assert_eq!(core2d.view.core_pipeline, CorePipelineKind::Core2d);
}

#[test]
fn orthographic_core3d_camera_keeps_an_orthographic_projection_matrix() {
    let camera = ViewportCameraSnapshot {
        core_pipeline: CorePipelineKind::Core3d,
        projection_mode: ProjectionMode::Orthographic,
        ..ViewportCameraSnapshot::default()
    };

    let matrix = ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(1280, 720))
        .clip_from_world_unjittered;

    assert_eq!(camera.core_pipeline_kind(), CorePipelineKind::Core3d);
    assert_eq!(matrix.w_axis.w, 1.0);
}

#[test]
fn scene_camera_asset_defaults_to_core3d_and_roundtrips_explicit_core2d() {
    let defaulted: SceneCameraAsset = toml::from_str("projection_mode = 'Orthographic'")
        .expect("deserialize camera asset without core pipeline");
    assert_eq!(defaulted.core_pipeline, CorePipelineKind::Core3d);
    assert_eq!(defaulted.projection_mode, ProjectionMode::Orthographic);

    let explicit = SceneCameraAsset {
        core_pipeline: CorePipelineKind::Core2d,
        projection_mode: ProjectionMode::Orthographic,
        ..SceneCameraAsset::default()
    };
    let document = toml::to_string(&explicit).expect("serialize explicit Core2d camera asset");
    let restored: SceneCameraAsset =
        toml::from_str(&document).expect("restore explicit Core2d camera asset");

    assert_eq!(restored, explicit);
    assert!(document.contains("core_pipeline"));
}

fn orthographic_extract(
    core_pipeline: CorePipelineKind,
) -> zircon_runtime::core::framework::render::RenderFrameExtract {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera);
    world
        .insert(
            camera,
            CameraComponent {
                core_pipeline,
                projection_mode: ProjectionMode::Orthographic,
                ..CameraComponent::default()
            },
        )
        .expect("insert camera component");

    world.build_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(1),
        SceneViewportExtractRequest {
            viewport_size: Some(UVec2::new(640, 360)),
            ..SceneViewportExtractRequest::default()
        },
    ))
}
