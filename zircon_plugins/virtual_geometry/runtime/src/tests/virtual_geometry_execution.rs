use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::test_support::render_feature_fixtures::{
    pluginized_wgpu_render_framework_with_gpu_test_collector,
    virtual_geometry_render_feature_descriptor, IndirectReadbackWords,
};
use zircon_runtime::asset::assets::{AlphaMode, MaterialAsset};
use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderLayerSet, RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

use crate::virtual_geometry::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
};
use zircon_runtime::graphics::{
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions,
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

fn virtual_geometry_scene_renderer(
    asset_manager: Arc<ProjectAssetManager>,
) -> crate::test_support::render_feature_fixtures::PluginizedWgpuRenderFrameworkFixture {
    crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_uploader_collector(
        asset_manager, Arc::new(crate::virtual_geometry::VirtualGeometryUploaderTestCollector::default()),
    )
}

fn capture_gpu_authority_words(
    asset_manager: Arc<ProjectAssetManager>,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: VirtualGeometryPrepareFrame,
    compiled: &CompiledRenderPipeline,
) -> IndirectReadbackWords {
    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let (_debug, _rgba, words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract,
            Some(prepare),
            compiled,
        )
        .unwrap();
    words
}

/// Projects only fields physically emitted by the current GPU authority channel, in the order
/// encoded by the GPU-produced indirect args. Legacy execution buffers are not product graph
/// resources in this plugin version, so these tests must not pretend to drop them.
fn gpu_authority_rows_in_args_order(words: &IndirectReadbackWords) -> Vec<[u32; 14]> {
    let command_count = words.args.first().copied().unwrap_or_default() as usize;
    let tokens = words
        .args
        .get(1..1 + command_count.saturating_mul(4))
        .unwrap_or_default()
        .chunks_exact(4)
        .filter(|args| args[1] != 0)
        .map(|args| args[3])
        .collect::<Vec<_>>();
    let authority = words
        .authority
        .chunks_exact(14)
        .filter(|row| row[3] != 0)
        .map(|row| row.try_into().unwrap())
        .collect::<Vec<[u32; 14]>>();
    tokens
        .into_iter()
        .filter_map(|token| {
            authority
                .iter()
                .copied()
                .find(|row| ((row[8] << 16) | row[9]) == token)
        })
        .collect()
}

fn gpu_submission_records_with_tokens(words: &IndirectReadbackWords) -> Vec<(u32, u32, u32, u32)> {
    gpu_authority_rows_in_args_order(words)
        .into_iter()
        .map(|row| (row[1], row[3], row[8], row[9]))
        .collect()
}

fn gpu_execution_draw_ref_records(words: &IndirectReadbackWords) -> Vec<(u32, u32, u32, u32, u32)> {
    gpu_authority_rows_in_args_order(words)
        .into_iter()
        .map(|row| (row[6], row[1], row[3], row[8], row[9]))
        .collect()
}

fn gpu_submission_records_with_instances(
    words: &IndirectReadbackWords,
) -> Vec<(Option<u32>, u32, u32, u32, u32)> {
    gpu_authority_rows_in_args_order(words)
        .into_iter()
        .map(|row| (Some(row[0]), row[1], row[3], row[8], row[9]))
        .collect()
}

fn gpu_execution_segment_authority(
    words: &IndirectReadbackWords,
) -> Vec<(Option<u32>, u32, u32, u32, u32, u32, u32, u32, u32)> {
    gpu_authority_rows_in_args_order(words)
        .into_iter()
        .map(|row| {
            (
                Some(row[0]),
                row[1],
                row[5],
                row[3],
                row[10],
                row[12],
                row[13],
                row[8],
                row[9],
            )
        })
        .collect()
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
            .join("transparent_red.zmaterial"),
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
            .join("opaque_white.zmaterial"),
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
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
        .render_compiled_virtual_geometry_test_frame_with_selections(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
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
            .join("transparent_red.zmaterial"),
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
            .join("opaque_white.zmaterial"),
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
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
        .render_compiled_virtual_geometry_test_frame_with_selections(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
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
            .join("transparent_red.zmaterial"),
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
            .join("opaque_white.zmaterial"),
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
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
        .render_compiled_virtual_geometry_test_frame_with_selections(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
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
            .join("transparent_red.zmaterial"),
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
            .join("opaque_white.zmaterial"),
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
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
        .render_compiled_virtual_geometry_test_frame_with_selections(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
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
            .join("transparent_red.zmaterial"),
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
            .join("opaque_white.zmaterial"),
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
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
        .render_compiled_virtual_geometry_test_frame_with_selections(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
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
            .join("transparent_red.zmaterial"),
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
            .join("opaque_white.zmaterial"),
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
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
        .render_compiled_virtual_geometry_test_frame_with_selections(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
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
                    stable_instance_key: materials[0].0 << 16,
                    transform_revision: 0,
                    transform: Transform {
                        translation: Vec3::ZERO,
                        scale: Vec3::new(0.8, 0.8, 1.0),
                        ..Transform::default()
                    },
                    model: model.clone(),
                    mesh: None,
                    material: materials[0].1,
                    mesh_lod: None,
                    morph_weights: Vec::new(),
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    static_state: Default::default(),
                    common: RendererCommon {
                        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                            default_render_layer_mask(),
                        ),
                        is_static: false,
                        ..RendererCommon::default()
                    },
                },
                RenderMeshSnapshot {
                    node_id: materials[1].0,
                    stable_instance_key: materials[1].0 << 16,
                    transform_revision: 0,
                    transform: Transform {
                        translation: Vec3::ZERO,
                        scale: Vec3::new(0.8, 0.8, 1.0),
                        ..Transform::default()
                    },
                    model,
                    mesh: None,
                    material: materials[1].1,
                    mesh_lod: None,
                    morph_weights: Vec::new(),
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    static_state: Default::default(),
                    common: RendererCommon {
                        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                            default_render_layer_mask(),
                        ),
                        is_static: false,
                        ..RendererCommon::default()
                    },
                },
            ],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        environment: Default::default(),
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
        page_dependencies: Vec::new(),
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
        &path,
        "v -1.0 -1.0 0.0\nv 1.0 -1.0 0.0\nv 1.0 1.0 0.0\nv -1.0 1.0 0.0\nvt 0.0 0.0\nvt 1.0 0.0\nvt 1.0 1.0\nvt 0.0 1.0\nvn 0.0 0.0 1.0\nf 1/1/1 2/2/1 3/3/1\nf 1/1/1 3/3/1 4/4/1\n",
    )
    .unwrap();
    // Use the normal project import transaction and real OBJ VG cook. The OBJ
    // vertices/indices above remain the shared source for both GPU mesh upload
    // and immutable VG primitive admission.
    let mut metadata = zircon_runtime::asset::project::AssetMetaDocument::new(
        zircon_runtime::asset::AssetUuid::new(),
        AssetUri::parse("res://models/quad.obj").unwrap(),
        zircon_runtime::asset::AssetKind::Model,
    );
    metadata.import_settings = "[virtual_geometry]\nenabled = true\n".parse().unwrap();
    let mut sidecar_name = path.file_name().unwrap().to_os_string();
    sidecar_name.push(".zmeta");
    metadata.save(path.with_file_name(sidecar_name)).unwrap();
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
        parent: None,
        options: Default::default(),
        queue: None,
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
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    fs::write(path, toml::to_string_pretty(&material).unwrap()).unwrap();
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

#[test]
fn virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order() {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_deferred_execution_source_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryDeferredExecutionSourceOrder",
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
            .join("transparent_red.zmaterial"),
        "DeferredTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.zmaterial"),
        "DeferredOpaqueWhite",
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    // The current compiled graph has no separate execution-args/offset buffer. Read the real GPU
    // indirect args and authority words, and preserve the original draw-ref and submission-order
    // assertions over those current product outputs.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_execution_draw_ref_records(&words),
        vec![(1, 3, 301, 1, 0), (0, 2, 300, 0, 0)],
        "expected actual GPU authority to follow deferred scene-pass submission order, with the opaque draw before the earlier transparent visibility slot"
    );
    assert_eq!(
        gpu_submission_records_with_tokens(&words),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected GPU authority submission records to preserve actual deferred order and visibility tokens"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_deferred_execution_records_survive_without_shared_indirect_buffers() {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_deferred_execution_records_only");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryDeferredExecutionRecordsOnly",
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
            .join("transparent_red.zmaterial"),
        "DeferredRecordsTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.zmaterial"),
        "DeferredRecordsOpaqueWhite",
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    // Legacy shared/execution buffer-drop hooks are absent from the current graph. The current
    // product source is the actual staged GPU args+authority pair.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_submission_records_with_tokens(&words),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected GPU authority to retain deferred submission records and their original visibility tokens"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_execution_segments_survive_without_shared_segment_and_draw_ref_buffers() {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_execution_segments_records_only");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionSegmentsRecordsOnly",
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
            .join("transparent_red.zmaterial"),
        "ExecutionSegmentsTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.zmaterial"),
        "ExecutionSegmentsOpaqueWhite",
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    // The current GPU authority is the product graph's segment source; separate shared and
    // execution segment buffers from the legacy observer are not produced here.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_execution_segment_authority(&words),
        vec![
            (Some(1), 3, 0, 301, 2, 0, 0, 1, 0),
            (Some(0), 2, 0, 300, 1, 0, 0, 0, 0),
        ],
        "expected GPU authority segment identities to preserve the original execution order, instance, page, slot, lineage, frontier, and token assertions"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone()
{
    let root =
        unique_temp_project_root("graphics_virtual_geometry_execution_records_recover_indices");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryExecutionRecordsRecoverIndices",
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
            .join("transparent_red.zmaterial"),
        "ExecutionRecordsTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.zmaterial"),
        "ExecutionRecordsOpaqueWhite",
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    // Execution-index buffers are absent from the current graph. Resolve draw-ref indices by
    // joining the real GPU first_instance tokens to their GPU authority records.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_execution_draw_ref_records(&words)
            .into_iter()
            .map(|record| record.0)
            .collect::<Vec<_>>(),
        vec![1, 0],
        "expected GPU authority joins to resolve actual submitted draw-ref indices in execution order"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_submission_records_survive_with_execution_indices_and_gpu_authority_buffer_only(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_gpu_authority_submission_records");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryGpuAuthoritySubmissionRecords",
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
            .join("transparent_red.zmaterial"),
        "GpuAuthorityTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.zmaterial"),
        "GpuAuthorityOpaqueWhite",
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    // The current compiled graph has no independent execution-record or shared-buffer fallback
    // chain; preserve the original ownership contract over the actual GPU authority output.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_submission_records_with_instances(&words),
        vec![(Some(1), 3, 301, 1, 0), (Some(0), 2, 300, 0, 0)],
        "expected actual GPU authority to preserve submission records and per-instance ownership in deferred order"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_shared_indirect_segments_preserve_instance_index_for_submission_fallback() {
    let root = unique_temp_project_root(
        "graphics_virtual_geometry_shared_indirect_segments_with_instances",
    );
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySharedIndirectSegmentsWithInstances",
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
            .join("white.zmaterial"),
        "SharedSegmentsInstanceIndexWhite",
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
    let material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, material.clone()), (3, material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    // The current graph has no separate execution buffers; its GPU authority is the actual
    // segment source and carries instance, page, slot, lineage, frontier, and submission identity.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_execution_segment_authority(&words),
        vec![
            (Some(0), 2, 0, 300, 1, 0, 0, 0, 0),
            (Some(1), 3, 0, 301, 2, 0, 0, 1, 0),
        ],
        "expected GPU authority segment identity and instance ownership to survive in the real shared source"
    );
    assert_eq!(
        gpu_submission_records_with_instances(&words),
        vec![(Some(0), 2, 300, 0, 0), (Some(1), 3, 301, 1, 0)],
        "expected the same GPU authority source to preserve instance ownership in submission records"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_execution_records_survive_with_execution_indices_and_gpu_authority_buffer_only()
{
    let root =
        unique_temp_project_root("graphics_virtual_geometry_gpu_authority_execution_records");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryGpuAuthorityExecutionRecords",
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
            .join("transparent_red.zmaterial"),
        "GpuAuthorityExecTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.zmaterial"),
        "GpuAuthorityExecOpaqueWhite",
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    // The current graph has no independent execution-index/record resources; assert the GPU
    // authority-derived record projection in the original deferred order.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_execution_draw_ref_records(&words),
        vec![(1, 3, 301, 1, 0), (0, 2, 300, 0, 0)],
        "expected actual GPU authority to retain draw-ref, entity, page, and visibility identity in deferred order"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_execution_segments_survive_with_execution_indices_and_gpu_authority_buffer_only(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_gpu_authority_execution_segments");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryGpuAuthorityExecutionSegments",
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
            .join("transparent_red.zmaterial"),
        "GpuAuthoritySegTransparentRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.zmaterial"),
        "GpuAuthoritySegOpaqueWhite",
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
        "res://materials/transparent_red.zmaterial",
    );
    let opaque_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/opaque_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, transparent_material), (3, opaque_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_deferred_pipeline(&extract);

    // The current graph exposes no independent execution-index/segment resources. Verify the
    // actual GPU authority segment identity fields in the compiled deferred frame.
    let words = capture_gpu_authority_words(
        asset_manager,
        viewport_size,
        extract,
        prepare_frame((2, 1), (3, 2)),
        &compiled,
    );
    assert_eq!(
        gpu_execution_segment_authority(&words),
        vec![
            (Some(1), 3, 0, 301, 2, 0, 0, 1, 0),
            (Some(0), 2, 0, 300, 1, 0, 0, 0, 0),
        ],
        "expected actual GPU authority to preserve deferred segment order and per-instance ownership"
    );

    let _ = fs::remove_dir_all(root);
}

fn compile_virtual_geometry_forward_plus_pipeline(
    extract: &RenderFrameExtract,
) -> CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
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

#[test]
fn virtual_geometry_renderer_submission_records_keep_draw_level_tokens_without_gpu_submission_buffer(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_submission_record_tokens_direct");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySubmissionRecordTokensDirect",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_unified_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_unified_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_two_primitive_gltf(
        paths
            .assets_root()
            .join("models")
            .join("double_triangle.gltf"),
    );
    write_unified_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
        "res://textures/white.png",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/double_triangle.gltf");
    let green_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_green.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract(viewport_size, model, green_material);
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport = renderer
        .create_virtual_geometry_test_viewport(viewport_size)
        .unwrap();
    renderer
        .submit_compiled_virtual_geometry_at_viewport(
            viewport,
            extract,
            Some(VirtualGeometryPrepareFrame {
                visible_entities: vec![2],
                visible_clusters: vec![VirtualGeometryPrepareCluster {
                    entity: 2,
                    cluster_id: 2,
                    page_id: 300,
                    lod_level: 0,
                    resident_slot: Some(1),
                    state: VirtualGeometryPrepareClusterState::Resident,
                }],
                cluster_draw_segments: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareDrawSegment {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        resident_slot: Some(1),
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lineage_depth: 0,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                ],
                resident_pages: vec![VirtualGeometryPreparePage {
                    page_id: 300,
                    slot: 1,
                    size_bytes: 4096,
                }],
                pending_page_requests: Vec::new(),
                available_slots: Vec::new(),
                evictable_pages: Vec::new(),
            }),
            &compiled,
            None,
        )
        .unwrap();

    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(2, 300, 0, 0), (2, 300, 0, 1)],
        "expected renderer-side draw-level submission records to remain available even after the GPU submission-token buffer is gone, instead of collapsing renderer authority back to buffer-only observability"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_renderer_submission_records_fall_back_to_gpu_generated_indirect_args_tokens_when_debug_channels_are_missing(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_submission_record_tokens_args_only");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySubmissionRecordTokensArgsOnly",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_unified_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_unified_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_two_primitive_gltf(
        paths
            .assets_root()
            .join("models")
            .join("double_triangle.gltf"),
    );
    write_unified_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
        "res://textures/white.png",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/double_triangle.gltf");
    let green_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_green.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract(viewport_size, model, green_material);
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport = renderer
        .create_virtual_geometry_test_viewport(viewport_size)
        .unwrap();
    renderer
        .submit_compiled_virtual_geometry_at_viewport(
            viewport,
            extract,
            Some(VirtualGeometryPrepareFrame {
                visible_entities: vec![2],
                visible_clusters: vec![VirtualGeometryPrepareCluster {
                    entity: 2,
                    cluster_id: 2,
                    page_id: 300,
                    lod_level: 0,
                    resident_slot: Some(1),
                    state: VirtualGeometryPrepareClusterState::Resident,
                }],
                cluster_draw_segments: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareDrawSegment {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        resident_slot: Some(1),
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lineage_depth: 0,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                ],
                resident_pages: vec![VirtualGeometryPreparePage {
                    page_id: 300,
                    slot: 1,
                    size_bytes: 4096,
                }],
                pending_page_requests: Vec::new(),
                available_slots: Vec::new(),
                evictable_pages: Vec::new(),
            }),
            &compiled,
            None,
        )
        .unwrap();

    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(2, 300, 0, 0), (2, 300, 0, 1)],
        "expected renderer-side submission observability to reconstruct draw-level tokens directly from the real GPU-generated indirect args first_instance values once the dedicated debug submission channels are gone, instead of collapsing back to no token truth"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_unified_indirect_reconstructs_submission_records_from_gpu_authority_when_cpu_records_are_gone(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_gpu_authority_submission_records");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryGpuAuthoritySubmissionRecords",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_unified_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_unified_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_two_primitive_gltf(
        paths
            .assets_root()
            .join("models")
            .join("double_triangle.gltf"),
    );
    write_unified_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
        "res://textures/white.png",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/double_triangle.gltf");
    let material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_green.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract(viewport_size, model, material);
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport = renderer
        .create_virtual_geometry_test_viewport(viewport_size)
        .unwrap();
    renderer
        .submit_compiled_virtual_geometry_at_viewport(
            viewport,
            extract,
            Some(VirtualGeometryPrepareFrame {
                visible_entities: vec![2],
                visible_clusters: vec![VirtualGeometryPrepareCluster {
                    entity: 2,
                    cluster_id: 2,
                    page_id: 300,
                    lod_level: 0,
                    resident_slot: Some(1),
                    state: VirtualGeometryPrepareClusterState::Resident,
                }],
                cluster_draw_segments: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareDrawSegment {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        resident_slot: Some(1),
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lineage_depth: 0,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                ],
                resident_pages: vec![VirtualGeometryPreparePage {
                    page_id: 300,
                    slot: 1,
                    size_bytes: 4096,
                }],
                pending_page_requests: Vec::new(),
                available_slots: Vec::new(),
                evictable_pages: Vec::new(),
            }),
            &compiled,
            None,
        )
        .unwrap();

    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(2, 300, 0, 0), (2, 300, 0, 1)],
        "expected renderer-side submission observability to reconstruct draw-level tokens and entity/page ownership directly from GPU args+draw-ref+segment truth once the CPU-side submission records are gone, instead of collapsing back to no visibility-owned authority"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_unified_indirect_keeps_pending_request_frontier_rank_in_gpu_submission_and_indirect_args(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_frontier_rank_indirect");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFrontierRankIndirect",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_unified_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_unified_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_tiled_quad_obj(paths.assets_root().join("models").join("tiled_quad.obj"));
    write_unified_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
        "res://textures/white.png",
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/tiled_quad.obj");
    let green_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_green.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract(viewport_size, model, green_material);
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport = renderer
        .create_virtual_geometry_test_viewport(viewport_size)
        .unwrap();
    renderer
        .submit_compiled_virtual_geometry_at_viewport(
            viewport,
            extract.clone(),
            Some(VirtualGeometryPrepareFrame {
                visible_entities: vec![2],
                visible_clusters: vec![VirtualGeometryPrepareCluster {
                    entity: 2,
                    cluster_id: 2,
                    page_id: 300,
                    lod_level: 0,
                    resident_slot: None,
                    state: VirtualGeometryPrepareClusterState::PendingUpload,
                }],
                cluster_draw_segments: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareDrawSegment {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        resident_slot: None,
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lineage_depth: 0,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::PendingUpload,
                    },
                ],
                resident_pages: Vec::new(),
                pending_page_requests: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 1,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    },
                ],
                available_slots: Vec::new(),
                evictable_pages: Vec::new(),
            }),
            &compiled,
            None,
        )
        .unwrap();
    let early_frontier_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();
    let early_frontier_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();

    renderer
        .submit_compiled_virtual_geometry_at_viewport(
            viewport,
            extract,
            Some(VirtualGeometryPrepareFrame {
                visible_entities: vec![2],
                visible_clusters: vec![VirtualGeometryPrepareCluster {
                    entity: 2,
                    cluster_id: 2,
                    page_id: 300,
                    lod_level: 0,
                    resident_slot: None,
                    state: VirtualGeometryPrepareClusterState::PendingUpload,
                }],
                cluster_draw_segments: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareDrawSegment {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        resident_slot: None,
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lineage_depth: 0,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::PendingUpload,
                    },
                ],
                resident_pages: Vec::new(),
                pending_page_requests: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 2,
                        frontier_rank: 3,
                        assigned_slot: None,
                        recycled_page_id: None,
                    },
                    crate::virtual_geometry::VirtualGeometryPrepareRequest {
                        page_id: 301,
                        size_bytes: 4096,
                        generation: 3,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    },
                    crate::virtual_geometry::VirtualGeometryPrepareRequest {
                        page_id: 302,
                        size_bytes: 4096,
                        generation: 4,
                        frontier_rank: 1,
                        assigned_slot: None,
                        recycled_page_id: None,
                    },
                    crate::virtual_geometry::VirtualGeometryPrepareRequest {
                        page_id: 303,
                        size_bytes: 4096,
                        generation: 5,
                        frontier_rank: 2,
                        assigned_slot: None,
                        recycled_page_id: None,
                    },
                ],
                available_slots: Vec::new(),
                evictable_pages: Vec::new(),
            }),
            &compiled,
            None,
        )
        .unwrap();
    let late_frontier_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();
    let late_frontier_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();

    assert_ne!(
        early_frontier_args, late_frontier_args,
        "expected later pending request frontier rank to flow into the actual GPU indirect args so deeper cluster raster submission consumes runtime frontier order instead of only page/slot/lod/state"
    );
    assert_ne!(
        early_frontier_segments, late_frontier_segments,
        "expected the actual GPU-submitted segment buffer to preserve pending request frontier order instead of only preserving page/slot/state lineage metadata"
    );
    assert_eq!(
        late_frontier_segments,
        vec![(
            0,
            1,
            1,
            300,
            0,
            VirtualGeometryPrepareClusterState::PendingUpload,
            0,
            0,
            3,
        )],
        "expected the actual GPU-submitted indirect segment buffer to retain pending request frontier rank so runtime request order continues into real submission authority"
    );

    let _ = fs::remove_dir_all(root);
}

fn compile_virtual_geometry_pipeline(extract: &RenderFrameExtract) -> CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
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

fn build_single_entity_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
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
            meshes: vec![RenderMeshSnapshot {
                node_id: 2,
                stable_instance_key: 2 << 16,
                transform_revision: 0,
                transform: Transform {
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                model,
                mesh: None,
                material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                common: RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        default_render_layer_mask(),
                    ),
                    is_static: false,
                    ..RendererCommon::default()
                },
            }],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        environment: Default::default(),
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
        cluster_budget: 1,
        page_budget: 1,
        clusters: vec![RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: 2,
            hierarchy_node_id: None,
            page_id: 300,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        }],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    extract
}

fn write_two_primitive_gltf(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut binary = Vec::new();
    for value in [
        -1.0f32, -1.0, 0.0, //
        0.0, 1.0, 0.0, //
        -1.0, 1.0, 0.0,
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2] {
        binary.extend_from_slice(&index.to_le_bytes());
    }
    for value in [
        0.0f32, -1.0, 0.0, //
        1.0, 1.0, 0.0, //
        1.0, -1.0, 0.0,
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2] {
        binary.extend_from_slice(&index.to_le_bytes());
    }

    let buffer_path = path
        .ancestors()
        .nth(3)
        .expect("gltf path should live under <root>/assets/models")
        .join(".generated")
        .join("double_triangle.bin");
    if let Some(parent) = buffer_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&buffer_path, &binary).unwrap();

    let gltf = format!(
        r#"{{
  "asset": {{"version": "2.0"}},
  "buffers": [
    {{"uri": "../../.generated/double_triangle.bin", "byteLength": {}}}
  ],
  "bufferViews": [
    {{"buffer":0,"byteOffset":0,"byteLength":36,"target":34962}},
    {{"buffer":0,"byteOffset":36,"byteLength":12,"target":34963}},
    {{"buffer":0,"byteOffset":48,"byteLength":36,"target":34962}},
    {{"buffer":0,"byteOffset":84,"byteLength":12,"target":34963}}
  ],
  "accessors": [
    {{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-1.0,-1.0,0.0],"max":[0.0,1.0,0.0]}},
    {{"bufferView":1,"componentType":5125,"count":3,"type":"SCALAR","min":[0],"max":[2]}},
    {{"bufferView":2,"componentType":5126,"count":3,"type":"VEC3","min":[0.0,-1.0,0.0],"max":[1.0,1.0,0.0]}},
    {{"bufferView":3,"componentType":5125,"count":3,"type":"SCALAR","min":[0],"max":[2]}}
  ],
  "meshes": [
    {{
      "primitives": [
        {{"attributes":{{"POSITION":0}},"indices":1}},
        {{"attributes":{{"POSITION":2}},"indices":3}}
      ]
    }}
  ],
  "nodes": [
    {{"mesh": 0}}
  ],
  "scenes": [
    {{"nodes": [0]}}
  ],
  "scene": 0
}}"#,
        binary.len()
    );
    fs::write(&path, gltf).unwrap();
    enable_unified_virtual_geometry_cook(&path);
}

fn write_tiled_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        &path,
        "\
v -1.0 -1.0 0.0
v 0.0 -1.0 0.0
v 1.0 -1.0 0.0
v -1.0 0.0 0.0
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v -1.0 1.0 0.0
v 0.0 1.0 0.0
v 1.0 1.0 0.0
vt 0.0 1.0
vt 0.5 1.0
vt 1.0 1.0
vt 0.0 0.5
vt 0.5 0.5
vt 1.0 0.5
vt 0.0 0.0
vt 0.5 0.0
vt 1.0 0.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 5/5/1
f 1/1/1 5/5/1 4/4/1
f 2/2/1 3/3/1 6/6/1
f 2/2/1 6/6/1 5/5/1
f 4/4/1 5/5/1 8/8/1
f 4/4/1 8/8/1 7/7/1
f 5/5/1 6/6/1 9/9/1
f 5/5/1 9/9/1 8/8/1
",
    )
    .unwrap();
    enable_unified_virtual_geometry_cook(&path);
}

fn write_unified_flat_color_wgsl(path: PathBuf, color: [f32; 3]) {
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
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

struct VertexInput {{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {{
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    return output;
}}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>({:.6}, {:.6}, {:.6}, alpha) * model_data.tint;
}}
"#,
            color[0], color[1], color[2]
        ),
    )
    .unwrap();
}

fn write_unified_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    image::ImageBuffer::<image::Rgba<u8>, _>::from_fn(2, 2, |_x, _y| image::Rgba(rgba))
        .save_with_format(path, image::ImageFormat::Png)
        .unwrap();
}

fn write_unified_material(path: PathBuf, shader_uri: &str, texture_uri: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("VirtualGeometryUnifiedIndirect".to_string()),
        shader: asset_reference(shader_uri),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: Some(asset_reference(texture_uri)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn enable_unified_virtual_geometry_cook(path: &std::path::Path) {
    // Import the original mesh bytes through the normal VG cook transaction.
    let file_name = path.file_name().unwrap();
    let uri = format!("res://models/{}", file_name.to_str().unwrap());
    let mut metadata = zircon_runtime::asset::project::AssetMetaDocument::new(
        zircon_runtime::asset::AssetUuid::new(),
        AssetUri::parse(&uri).unwrap(),
        zircon_runtime::asset::AssetKind::Model,
    );
    metadata.import_settings = "[virtual_geometry]\nenabled = true\n".parse().unwrap();
    let mut sidecar_name = file_name.to_os_string();
    sidecar_name.push(".zmeta");
    metadata.save(path.with_file_name(sidecar_name)).unwrap();
}
