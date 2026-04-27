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
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryInstance,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use crate::graphics::tests::plugin_render_feature_fixtures::virtual_geometry_render_feature_descriptor;
use crate::scene::components::{default_render_layer_mask, Mobility};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::{
    types::{
        ViewportRenderFrame, VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    },
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

fn compile_virtual_geometry_pipeline(
    asset: RenderPipelineAsset,
    extract: &RenderFrameExtract,
) -> CompiledRenderPipeline {
    asset
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

fn compile_virtual_geometry_forward_plus_pipeline(
    extract: &RenderFrameExtract,
) -> CompiledRenderPipeline {
    compile_virtual_geometry_pipeline(RenderPipelineAsset::default_forward_plus(), extract)
}

fn compile_virtual_geometry_deferred_pipeline(
    extract: &RenderFrameExtract,
) -> CompiledRenderPipeline {
    compile_virtual_geometry_pipeline(RenderPipelineAsset::default_deferred(), extract)
}

fn virtual_geometry_scene_renderer(asset_manager: Arc<ProjectAssetManager>) -> SceneRenderer {
    SceneRenderer::new_with_plugin_render_features(
        asset_manager,
        [virtual_geometry_render_feature_descriptor()],
    )
    .unwrap()
}

#[test]
fn virtual_geometry_transparent_submission_order_follows_visibility_owned_indirect_authority() {
    let root = unique_temp_project_root("graphics_virtual_geometry_submission_execution_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySubmissionExecutionOrder",
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
            .join("red.material.toml"),
        "VisibilityOrderRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("blue.material.toml"),
        "VisibilityOrderBlue",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [0.0, 0.0, 1.0, 0.65],
        AlphaMode::Blend,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let red_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/red.material.toml");
    let blue_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/blue.material.toml");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, red_material), (3, blue_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let red_dominant_frame = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 2), (3, 1)))),
            &compiled,
            None,
        )
        .unwrap();
    let red_dominant_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let red_dominant_center = center_pixel(&red_dominant_frame.rgba, viewport_size);

    let blue_dominant_frame = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();
    let blue_dominant_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let blue_dominant_center = center_pixel(&blue_dominant_frame.rgba, viewport_size);

    assert_eq!(
        red_dominant_segments,
        vec![
            (
                0,
                1,
                1,
                301,
                1,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
            (
                0,
                1,
                1,
                300,
                2,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
        ],
        "expected the visibility-owned indirect segment order itself to swap when prepare-owned submission slots swap"
    );
    assert_eq!(
        blue_dominant_segments,
        vec![
            (
                0,
                1,
                1,
                300,
                1,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
            (
                0,
                1,
                1,
                301,
                2,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
        ],
        "expected the visibility-owned indirect segment order to flip with the authoritative submission-slot swap"
    );
    assert!(
        red_dominant_center[0] > red_dominant_center[2],
        "expected the center pixel to become red-dominant once entity 2 is authoritative last in the indirect submission order; got {:?}",
        red_dominant_center
    );
    assert!(
        blue_dominant_center[2] > blue_dominant_center[0],
        "expected the center pixel to become blue-dominant once entity 3 is authoritative last in the indirect submission order; got {:?}",
        blue_dominant_center
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_transparent_fallback_submission_order_follows_prepare_owned_unified_indirect_when_segments_are_absent(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_submission_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSubmissionOrder",
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
            .join("red.material.toml"),
        "FallbackVisibilityOrderRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("blue.material.toml"),
        "FallbackVisibilityOrderBlue",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [0.0, 0.0, 1.0, 0.65],
        AlphaMode::Blend,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let red_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/red.material.toml");
    let blue_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/blue.material.toml");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, red_material), (3, blue_material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let red_dominant_frame = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame_without_segments(
                    (2, 2),
                    (3, 1),
                ))),
            &compiled,
            None,
        )
        .unwrap();
    let red_dominant_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let red_dominant_center = center_pixel(&red_dominant_frame.rgba, viewport_size);

    let blue_dominant_frame = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame_without_segments(
                    (2, 1),
                    (3, 2),
                ))),
            &compiled,
            None,
        )
        .unwrap();
    let blue_dominant_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let blue_dominant_center = center_pixel(&blue_dominant_frame.rgba, viewport_size);

    assert_eq!(
        red_dominant_segments,
        vec![
            (
                0,
                1,
                1,
                301,
                1,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
            (
                0,
                1,
                1,
                300,
                2,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
        ],
        "expected missing explicit draw segments to reuse the same visibility-owned unified indirect ordering, so synthesized fallback cluster draws still enter real submission in authoritative slot order"
    );
    assert_eq!(
        blue_dominant_segments,
        vec![
            (
                0,
                1,
                1,
                300,
                1,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
            (
                0,
                1,
                1,
                301,
                2,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
        ],
        "expected swapping fallback submission slots without explicit cluster segments to flip the synthesized unified indirect order instead of falling back to fixed CPU mesh insertion order"
    );
    assert!(
        red_dominant_center[0] > red_dominant_center[2],
        "expected the center pixel to become red-dominant once missing-segment fallback slices still honor prepare-owned unified indirect order; got {:?}",
        red_dominant_center
    );
    assert!(
        blue_dominant_center[2] > blue_dominant_center[0],
        "expected the center pixel to become blue-dominant once fallback unified indirect authority flips; got {:?}",
        blue_dominant_center
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_gpu_generated_args_expose_visibility_owned_submission_index_in_first_instance()
{
    let root = unique_temp_project_root("graphics_virtual_geometry_gpu_args_submission_index");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryGpuArgsSubmissionIndex",
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
            .join("white.material.toml"),
        "GpuArgsSubmissionIndexWhite",
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
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/white.material.toml");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, material.clone()), (3, material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 2), (3, 1)))),
            &compiled,
            None,
        )
        .unwrap();

    let indirect_args = renderer
        .read_last_virtual_geometry_indirect_args_with_instances()
        .unwrap();

    assert_eq!(
        indirect_args,
        vec![(0, 6, 0), (0, 6, 1 << 16)],
        "expected the real GPU-generated indirect args source to encode visibility-owned submission tokens into first_instance so unified-indirect authority reaches the actual args buffer instead of staying on the parallel debug token side-channel"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_renderer_mesh_draw_submission_order_tracks_visibility_owned_unified_indirect_authority(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_renderer_submission_order_truth");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryRendererSubmissionOrderTruth",
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
            .join("white.material.toml"),
        "RendererSubmissionOrderWhite",
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
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/white.material.toml");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, material.clone()), (3, material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 2), (3, 1)))),
            &compiled,
            None,
        )
        .unwrap();
    let red_dominant_order =
        renderer.read_last_virtual_geometry_mesh_draw_submission_order_with_instances();

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(prepare_frame((2, 1), (3, 2)))),
            &compiled,
            None,
        )
        .unwrap();
    let blue_dominant_order =
        renderer.read_last_virtual_geometry_mesh_draw_submission_order_with_instances();

    assert_eq!(
        red_dominant_order,
        vec![(Some(1), 3, 301), (Some(0), 2, 300)],
        "expected renderer-side mesh draw submission order to preserve per-instance ownership alongside the same visibility-owned unified indirect order when entity 3 owns the earlier authoritative slot"
    );
    assert_eq!(
        blue_dominant_order,
        vec![(Some(0), 2, 300), (Some(1), 3, 301)],
        "expected renderer-side mesh draw submission order to flip with the same visibility-owned unified indirect authority while keeping the matching instance ownership instead of staying on fixed CPU mesh insertion order"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_deferred_execution_source_tracks_actual_scene_pass_submission_order() {
    const INDIRECT_ARGS_STRIDE_BYTES: u64 = (std::mem::size_of::<u32>() as u64) * 5;

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
            .join("transparent_red.material.toml"),
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
            .join("opaque_white.material.toml"),
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

    assert_eq!(
        renderer.read_last_virtual_geometry_execution_indirect_offsets(),
        vec![0, INDIRECT_ARGS_STRIDE_BYTES],
        "expected the actual deferred scene-pass draws to consume a compact execution-owned indirect args source instead of keeping shared visibility-slot offsets"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_indirect_execution_draw_ref_indices()
            .unwrap(),
        vec![1, 0],
        "expected the GPU execution subset source to follow actual deferred scene-pass submission order, with the opaque draw executing before the earlier transparent unified-indirect slot"
    );

    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected deepest fallback submission observability to preserve actual deferred scene-pass execution order instead of re-sorting back to visibility-slot token order"
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
            .join("transparent_red.material.toml"),
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
            .join("opaque_white.material.toml"),
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

    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_segments_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(3, 301, 1, 0), (2, 300, 0, 0)],
        "expected deferred execution truth to survive even after shared indirect args, draw-ref, and segment buffers are gone, instead of collapsing submission observability back to an empty result"
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
            .join("transparent_red.material.toml"),
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
            .join("opaque_white.material.toml"),
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

    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_segments_buffer_for_test();

    let execution_segments = renderer
        .read_last_virtual_geometry_indirect_execution_segments_with_entities()
        .unwrap();
    assert_eq!(
        execution_segments
            .iter()
            .map(|segment| {
                (
                    segment.instance_index,
                    (
                        segment.entity,
                        segment.cluster_start_ordinal,
                        segment.cluster_span_count,
                        segment.cluster_total_count,
                        segment.page_id,
                        segment.submission_slot,
                        segment.state,
                        segment.lineage_depth,
                        segment.lod_level,
                        segment.frontier_rank,
                        segment.submission_index,
                        segment.draw_ref_rank,
                    ),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                Some(1),
                (
                    3,
                    0,
                    1,
                    1,
                    301,
                    2,
                    VirtualGeometryPrepareClusterState::Resident,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
            ),
            (
                Some(0),
                (
                    2,
                    0,
                    1,
                    1,
                    300,
                    1,
                    VirtualGeometryPrepareClusterState::Resident,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
            ),
        ],
        "expected actual executed cluster-raster segment truth to survive on the deeper execution-record source with per-instance ownership intact even after shared segment and draw-ref buffers are gone"
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
            .join("transparent_red.material.toml"),
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
            .join("opaque_white.material.toml"),
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

    renderer.drop_last_virtual_geometry_indirect_execution_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_indirect_execution_draw_ref_indices()
            .unwrap(),
        vec![1, 0],
        "expected the deeper execution-record source to reconstruct actual submitted draw-ref indices even when the dedicated execution index buffer is gone"
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
            .join("transparent_red.material.toml"),
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
            .join("opaque_white.material.toml"),
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
    renderer.drop_last_virtual_geometry_indirect_execution_records_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_segments_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_instances()
            .unwrap(),
        vec![(Some(1), 3, 301, 1, 0), (Some(0), 2, 300, 0, 0)],
        "expected actual submission records to survive on a GPU-generated authority source with the same per-instance ownership once CPU submission records, execution records, indirect args, submission tokens, draw-ref buffer, and segment buffer are all gone"
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
            .join("white.material.toml"),
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
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/white.material.toml");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, material.clone()), (3, material)],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

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
    renderer.drop_last_virtual_geometry_indirect_execution_records_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_execution_authority_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_authority_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_indirect_segments_with_instances()
            .unwrap(),
        vec![
            (
                Some(0),
                2,
                0,
                1,
                1,
                300,
                1,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
                0,
            ),
            (
                Some(1),
                3,
                0,
                1,
                1,
                301,
                2,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
                1,
            ),
        ],
        "expected shared indirect segment readback itself to preserve per-instance ownership so later fallback helpers do not have to collapse back to entity-only tuples"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_instances()
            .unwrap(),
        vec![(Some(0), 2, 300, 0, 0), (Some(1), 3, 301, 1, 0)],
        "expected submission-record fallback to reuse shared segment instance ownership once execution authority, execution args, and CPU mirrors are all gone"
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
            .join("transparent_red.material.toml"),
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
            .join("opaque_white.material.toml"),
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

    renderer.drop_last_virtual_geometry_indirect_execution_records_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_segments_buffer_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_indirect_execution_records()
            .unwrap(),
        vec![(1, 3, 301, 1, 0), (0, 2, 300, 0, 0)],
        "expected actual execution records to survive on execution indices + GPU authority alone once the host-built execution records buffer and older shared indirect buffers are gone"
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
            .join("transparent_red.material.toml"),
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
            .join("opaque_white.material.toml"),
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

    renderer.drop_last_virtual_geometry_indirect_execution_records_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_args_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_draw_refs_buffer_for_test();
    renderer.drop_last_virtual_geometry_indirect_segments_buffer_for_test();

    let execution_segments = renderer
        .read_last_virtual_geometry_indirect_execution_segments_with_entities()
        .unwrap();
    assert_eq!(
        execution_segments
            .iter()
            .map(|segment| {
                (
                    segment.instance_index,
                    (
                        segment.entity,
                        segment.cluster_start_ordinal,
                        segment.cluster_span_count,
                        segment.cluster_total_count,
                        segment.page_id,
                        segment.submission_slot,
                        segment.state,
                        segment.lineage_depth,
                        segment.lod_level,
                        segment.frontier_rank,
                        segment.submission_index,
                        segment.draw_ref_rank,
                    ),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                Some(1),
                (
                    3,
                    0,
                    1,
                    1,
                    301,
                    2,
                    VirtualGeometryPrepareClusterState::Resident,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
            ),
            (
                Some(0),
                (
                    2,
                    0,
                    1,
                    1,
                    300,
                    1,
                    VirtualGeometryPrepareClusterState::Resident,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
            ),
        ],
        "expected actual execution segments to survive on execution indices + GPU authority alone with the same per-instance ownership that the execution snapshot already exposes"
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
        instances: vec![
            RenderVirtualGeometryInstance {
                entity: materials[0].0,
                source_model: None,
                transform: Transform {
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                cluster_offset: 0,
                cluster_count: 1,
                page_offset: 0,
                page_count: 1,
                mesh_name: Some("SubmissionExecutionOrderMesh0".to_string()),
                source_hint: Some("graphics-test".to_string()),
            },
            RenderVirtualGeometryInstance {
                entity: materials[1].0,
                source_model: None,
                transform: Transform {
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                cluster_offset: 1,
                cluster_count: 1,
                page_offset: 1,
                page_count: 1,
                mesh_name: Some("SubmissionExecutionOrderMesh1".to_string()),
                source_hint: Some("graphics-test".to_string()),
            },
        ],
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

fn prepare_frame_without_segments(
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
        cluster_draw_segments: Vec::new(),
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

fn center_pixel(rgba: &[u8], size: UVec2) -> [u8; 4] {
    let x = (size.x / 2) as usize;
    let y = (size.y / 2) as usize;
    let index = (y * size.x as usize + x) * 4;
    [
        rgba[index],
        rgba[index + 1],
        rgba[index + 2],
        rgba[index + 3],
    ]
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

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba(rgba))
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
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
    base_color: [f32; 4],
    alpha_mode: AlphaMode,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some(name.to_string()),
        shader: asset_reference(shader_uri),
        base_color,
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
