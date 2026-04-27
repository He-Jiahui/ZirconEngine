use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::assets::{AlphaMode, MaterialAsset};
use crate::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use crate::graphics::tests::plugin_render_feature_fixtures::virtual_geometry_render_feature_descriptor;
use crate::scene::components::{default_render_layer_mask, Mobility};

use crate::{
    types::{
        ViewportRenderFrame, VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    },
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

fn compile_virtual_geometry_deferred_pipeline(
    extract: &RenderFrameExtract,
) -> CompiledRenderPipeline {
    RenderPipelineAsset::default_deferred()
        .with_plugin_render_features([virtual_geometry_render_feature_descriptor()])
        .compile_with_options(
            extract,
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_async_compute(false),
        )
        .unwrap()
}

fn virtual_geometry_scene_renderer(asset_manager: Arc<ProjectAssetManager>) -> SceneRenderer {
    SceneRenderer::new_with_plugin_render_features(
        asset_manager,
        [virtual_geometry_render_feature_descriptor()],
    )
    .unwrap()
}

#[test]
fn virtual_geometry_submission_records_survive_with_execution_args_and_gpu_authority_only() {
    let root = unique_temp_project_root("graphics_virtual_geometry_execution_args_authority");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionArgsAuthority",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("transparent_red.material.toml"),
        "ExecutionArgsTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.material.toml"),
        "ExecutionArgsOpaqueWhite",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let transparent_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/transparent_red.material.toml",
    );
    let opaque_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/opaque_white.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );
    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();

    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_records_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_segments_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected actual deferred submission truth to survive on a deeper GPU-generated execution-args source plus authority buffer once host execution-index/record mirrors and shared indirect superset buffers are gone"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_submission_records_survive_with_execution_args_and_shared_submission_tokens_only(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_execution_args_submission_tokens");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionArgsSubmissionTokens",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("transparent_red.material.toml"),
        "ExecutionArgsTokensTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.material.toml"),
        "ExecutionArgsTokensOpaqueWhite",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let transparent_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/transparent_red.material.toml",
    );
    let opaque_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/opaque_white.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );
    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();

    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_records_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_authority_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected actual deferred submission truth to survive on execution args plus shared submission-token / draw-ref / segment buffers even after the authority sidecar is removed"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_submission_records_survive_with_execution_authority_buffer_only() {
    let root = unique_temp_project_root("graphics_virtual_geometry_execution_authority_only");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionAuthorityOnly",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("transparent_red.material.toml"),
        "ExecutionAuthorityOnlyTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.material.toml"),
        "ExecutionAuthorityOnlyOpaqueWhite",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let transparent_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/transparent_red.material.toml",
    );
    let opaque_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/opaque_white.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );
    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();

    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_records_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_authority_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_segments_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected actual deferred submission truth to survive on execution-owned compact authority records even after the shared visibility-owned authority and remap buffers are gone"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_submission_records_survive_with_execution_submission_tokens_and_shared_authority_only(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_execution_submission_tokens_only");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionSubmissionTokensOnly",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("transparent_red.material.toml"),
        "ExecutionSubmissionTokensTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.material.toml"),
        "ExecutionSubmissionTokensOpaqueWhite",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let transparent_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/transparent_red.material.toml",
    );
    let opaque_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/opaque_white.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );
    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();

    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_authority_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected actual deferred submission truth to survive on a compact execution-owned submission-token source plus shared authority after execution args and shared token/args buffers are gone"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_execution_draw_ref_indices_default_to_execution_args_without_dedicated_execution_buffer(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_execution_args_default");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionArgsDefault",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("transparent_red.material.toml"),
        "ExecutionArgsDefaultTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.material.toml"),
        "ExecutionArgsDefaultOpaqueWhite",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let transparent_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/transparent_red.material.toml",
    );
    let opaque_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/opaque_white.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );
    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();

    assert!(
        !renderer.has_last_virtual_geometry_indirect_execution_buffer_for_test(),
        "expected actual execution draw-ref recovery to default to execution-args sources instead of publishing a dedicated host-built execution-index buffer"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_indirect_execution_draw_ref_indices()
            .unwrap(),
        vec![1, 0],
        "expected draw-ref recovery to stay on the actual deferred execution order after removing the dedicated execution-index sidecar"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_execution_records_default_to_execution_args_and_authority_without_dedicated_execution_records_buffer(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_execution_records_default");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionRecordsDefault",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("transparent_red.material.toml"),
        "ExecutionRecordsDefaultTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.material.toml"),
        "ExecutionRecordsDefaultOpaqueWhite",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let transparent_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/transparent_red.material.toml",
    );
    let opaque_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/opaque_white.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );
    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();

    assert!(
        !renderer.has_last_virtual_geometry_indirect_execution_records_buffer_for_test(),
        "expected actual execution record recovery to default to execution-args + authority instead of publishing a dedicated host-built execution-records buffer"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_indirect_execution_records()
            .unwrap(),
        vec![(1, 3, 301, 1, 0), (0, 2, 300, 0, 0)],
        "expected execution-record readback to stay on the actual deferred execution order after removing the dedicated execution-records sidecar"
    );

    let _ = fs::remove_dir_all(root);
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_graphics_{label}_{unique}"))
}

fn build_overlapping_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    materials: [(u64, ResourceHandle<MaterialMarker>); 2],
    clusters: Vec<RenderVirtualGeometryCluster>,
    pages: Vec<RenderVirtualGeometryPage>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 1.2,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                RenderMeshSnapshot {
                    node_id: materials[0].0,
                    transform: Transform {
                        translation: Vec3::ZERO,
                        scale: Vec3::new(0.8, 0.8, 1.0),
                        ..Transform::default()
                    },
                    model: model.clone(),
                    material: materials[0].1,
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
                RenderMeshSnapshot {
                    node_id: materials[1].0,
                    transform: Transform {
                        translation: Vec3::ZERO,
                        scale: Vec3::new(0.8, 0.8, 1.0),
                        ..Transform::default()
                    },
                    model,
                    material: materials[1].1,
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
            ],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: clusters.len() as u32,
        page_budget: pages.len() as u32,
        clusters,
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages,
        instances: Vec::new(),
        debug: Default::default(),
    });
    extract
}

fn prepare_frame(
    first_entity_slot: (u64, u32),
    second_entity_slot: (u64, u32),
) -> VirtualGeometryPrepareFrame {
    VirtualGeometryPrepareFrame {
        visible_entities: vec![first_entity_slot.0, second_entity_slot.0],
        visible_clusters: vec![
            VirtualGeometryPrepareCluster {
                entity: first_entity_slot.0,
                cluster_id: 20,
                page_id: 300,
                lod_level: 0,
                resident_slot: Some(first_entity_slot.1),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareCluster {
                entity: second_entity_slot.0,
                cluster_id: 30,
                page_id: 301,
                lod_level: 0,
                resident_slot: Some(second_entity_slot.1),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        cluster_draw_segments: vec![
            draw_segment(first_entity_slot.0, 20, 300, first_entity_slot.1),
            draw_segment(second_entity_slot.0, 30, 301, second_entity_slot.1),
        ],
        resident_pages: vec![
            VirtualGeometryPreparePage {
                page_id: 300,
                slot: first_entity_slot.1,
                size_bytes: 4096,
            },
            VirtualGeometryPreparePage {
                page_id: 301,
                slot: second_entity_slot.1,
                size_bytes: 4096,
            },
        ],
        pending_page_requests: Vec::new(),
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    }
}

fn draw_segment(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    resident_slot: u32,
) -> VirtualGeometryPrepareDrawSegment {
    VirtualGeometryPrepareDrawSegment {
        entity,
        cluster_id,
        page_id,
        resident_slot: Some(resident_slot),
        cluster_ordinal: 0,
        cluster_span_count: 1,
        cluster_count: 1,
        lineage_depth: 0,
        lod_level: 0,
        state: VirtualGeometryPrepareClusterState::Resident,
    }
}

fn cluster(entity: u64, cluster_id: u32, page_id: u32) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level: 0,
        parent_cluster_id: None,
        bounds_center: Vec3::ZERO,
        bounds_radius: 1.0,
        screen_space_error: 1.0,
    }
}

fn page(page_id: u32) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident: true,
        size_bytes: 4096,
    }
}

fn write_flat_color_wgsl(path: PathBuf, color: [f32; 3]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        format!(
            r#"
struct SceneUniform {{
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
}};
struct ModelUniform {{
    model: mat4x4<f32>,
    tint: vec4<f32>,
}};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var color_texture: texture_2d<f32>;
@group(2) @binding(1) var color_sampler: sampler;

struct VertexInput {{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOutput {{
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {{
    var out: VertexOutput;
    out.position = scene.view_proj * model.model * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    return out;
}}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    let tex = textureSample(color_texture, color_sampler, input.uv);
    return vec4<f32>({0}, {1}, {2}, tex.a) * model.tint;
}}
"#,
            color[0], color[1], color[2]
        ),
    )
    .unwrap();
}

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let image = image::ImageBuffer::from_fn(1, 1, |_x, _y| image::Rgba(rgba));
    image.save(path).unwrap();
}

fn write_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "v -1.0 -1.0 0.0\nv 1.0 -1.0 0.0\nv 1.0 1.0 0.0\nv -1.0 1.0 0.0\nvt 0.0 0.0\nvt 1.0 0.0\nvt 1.0 1.0\nvt 0.0 1.0\nvn 0.0 0.0 1.0\nf 1/1/1 2/2/1 3/3/1\nf 1/1/1 3/3/1 4/4/1\n",
    )
    .unwrap();
}

fn write_material(
    path: PathBuf,
    name: &str,
    shader_uri: &str,
    texture_uri: &str,
    tint: [f32; 4],
    alpha_mode: AlphaMode,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some(name.to_string()),
        shader: asset_reference(shader_uri),
        base_color: tint,
        base_color_texture: Some(asset_reference(texture_uri)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode,
        double_sided: false,
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource id for {uri}")),
    )
}
