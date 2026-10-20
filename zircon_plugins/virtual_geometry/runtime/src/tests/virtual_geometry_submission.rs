use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::assets::{AlphaMode, MaterialAsset};
use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderLayerSet, RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

use crate::{
    test_support::render_feature_fixtures::{
        pluginized_wgpu_render_framework_with_gpu_test_collector,
        virtual_geometry_render_feature_descriptor, IndirectReadbackWords,
    },
    virtual_geometry::{
        VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
        VirtualGeometryPrepareRequest,
    },
};
use zircon_runtime::graphics::{
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions,
};

fn compile_virtual_geometry_pipeline(extract: &RenderFrameExtract) -> CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([virtual_geometry_render_feature_descriptor()])
        .compile_with_options(
            extract,
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::Temporal)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_async_compute(false),
        )
        .unwrap()
}

fn gpu_authority_rows_in_args_order(words: &IndirectReadbackWords) -> Vec<[u32; 14]> {
    let rows = words
        .args
        .get(1..)
        .unwrap_or_default()
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, args)| args[0] != 0)
        .filter_map(|(slot, _)| {
            words
                .authority
                .chunks_exact(14)
                .nth(slot)
                .map(|row| row.try_into().expect("GPU authority row is 14 words"))
        })
        .collect::<Vec<[u32; 14]>>();
    assert_eq!(
        words.args.first().copied().unwrap_or_default() as usize,
        rows.len(),
        "GPU indirect count must match the actual nonzero argument slots"
    );
    rows
}

fn gpu_args_in_submission_order(words: &IndirectReadbackWords) -> Vec<(u32, u32)> {
    words
        .args
        .get(1..)
        .unwrap_or_default()
        .chunks_exact(4)
        .filter(|args| args[0] != 0)
        .map(|args| (args[0], args[3]))
        .collect()
}

#[test]
fn virtual_geometry_unified_indirect_uses_fallback_recycle_slot_authority_for_submission_order_and_draw_refs(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_slot_submission");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSlotSubmission",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_tiled_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("tiled_quad.obj"),
        4,
    );
    write_material(
        paths
            .asset_root(&RelPath::project_assets())
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
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![cluster(2, 20, 300, 0), cluster(2, 30, 301, 1)],
        vec![
            page(300, false),
            page(301, false),
            page(400, true),
            page(800, true),
        ],
    );
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let ((colder_debug, _colder_rgba, colder_words), (hotter_debug, _hotter_rgba, hotter_words)) =
        fixture
            .render_compiled_virtual_geometry_captured_test_frame_pair(
                viewport_size,
                (extract.clone(), Some(dual_pending_prepare_frame(800, 400))),
                (extract, Some(dual_pending_prepare_frame(400, 800))),
                &compiled,
            )
            .unwrap();
    let colder_segments = colder_debug
        .as_ref()
        .expect("first submitted frame exposes its execution segments")
        .execution_segments
        .as_slice();
    let hotter_segments = hotter_debug
        .as_ref()
        .expect("second submitted frame exposes its execution segments")
        .execution_segments
        .as_slice();
    let colder_authority = gpu_authority_rows_in_args_order(&colder_words);
    let hotter_authority = gpu_authority_rows_in_args_order(&hotter_words);

    assert_eq!(
        colder_segments
            .iter()
            .map(|segment| segment.page_id)
            .collect::<Vec<_>>(),
        vec![301, 300],
        "the public frame snapshot must retain fallback-slot submission ordering"
    );
    assert!(colder_segments
        .iter()
        .all(|segment| { segment.state == RenderVirtualGeometryExecutionState::PendingUpload }));
    assert_eq!(
        colder_authority.iter().map(|row| row[3]).collect::<Vec<_>>(),
        vec![301, 300],
        "actual GPU authority must submit the colder fallback page before the later slot-owned page"
    );
    assert_eq!(
        colder_authority
            .iter()
            .map(|row| row[6])
            .collect::<Vec<_>>(),
        vec![1, 0],
        "the GPU authority draw refs must remain paired with the reordered page submissions"
    );
    assert_eq!(
        hotter_segments
            .iter()
            .map(|segment| segment.page_id)
            .collect::<Vec<_>>(),
        vec![300, 301],
        "swapping fallback recycle preference must swap the frame snapshot's page order"
    );
    assert!(hotter_segments
        .iter()
        .all(|segment| { segment.state == RenderVirtualGeometryExecutionState::PendingUpload }));
    assert_eq!(
        hotter_authority
            .iter()
            .map(|row| row[3])
            .collect::<Vec<_>>(),
        vec![300, 301],
        "actual GPU authority must follow the swapped fallback page order"
    );
    assert_eq!(
        hotter_authority
            .iter()
            .map(|row| row[6])
            .collect::<Vec<_>>(),
        vec![0, 1],
        "the GPU authority draw refs must remain paired with the swapped page submissions"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_segment_buffer_keeps_prepare_owned_segments_when_some_entities_do_not_emit_pending_draws(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_prepare_owned_segments");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPrepareOwnedSegments",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material(
        paths
            .asset_root(&RelPath::project_assets())
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

    let valid_model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_green.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, valid_model),
        (3, valid_model),
        green_material,
        vec![cluster(2, 20, 300, 0), cluster(3, 30, 301, 0)],
        vec![page(300, true), page(301, true)],
    );
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let (snapshot, _rgba, words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract,
            Some(VirtualGeometryPrepareFrame {
                visible_entities: vec![2],
                visible_clusters: vec![
                    VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 20,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                    VirtualGeometryPrepareCluster {
                        entity: 3,
                        cluster_id: 30,
                        page_id: 301,
                        lod_level: 0,
                        resident_slot: Some(2),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                ],
                cluster_draw_segments: vec![
                    crate::virtual_geometry::VirtualGeometryPrepareDrawSegment {
                        entity: 2,
                        cluster_id: 20,
                        page_id: 300,
                        resident_slot: Some(1),
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lineage_depth: 0,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                    crate::virtual_geometry::VirtualGeometryPrepareDrawSegment {
                        entity: 3,
                        cluster_id: 30,
                        page_id: 301,
                        resident_slot: Some(2),
                        cluster_ordinal: 0,
                        cluster_span_count: 1,
                        cluster_count: 1,
                        lineage_depth: 0,
                        lod_level: 0,
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                ],
                resident_pages: vec![
                    VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    },
                    VirtualGeometryPreparePage {
                        page_id: 301,
                        slot: 2,
                        size_bytes: 4096,
                    },
                ],
                pending_page_requests: Vec::new(),
                available_slots: Vec::new(),
                evictable_pages: Vec::new(),
            }),
            &compiled,
        )
        .unwrap();
    let snapshot = snapshot.expect("submitted frame exposes its debug snapshot");
    let segments = &snapshot.execution_segments;
    let gpu_authority = gpu_authority_rows_in_args_order(&words);

    assert_eq!(
        segments.len(),
        2,
        "the submitted frame snapshot must retain both prepare-owned segments even when only one entity emits a GPU draw"
    );
    assert_eq!(
        segments.iter().map(|segment| segment.page_id).collect::<Vec<_>>(),
        vec![300, 301],
        "the debug snapshot must retain prepare-owned page ordering independently of drawable GPU commands"
    );
    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.entity)
            .collect::<Vec<_>>(),
        vec![2, 3],
        "the snapshot must retain both prepare-owned entity segments"
    );
    assert!(segments
        .iter()
        .all(|segment| segment.state == RenderVirtualGeometryExecutionState::Resident));
    assert_eq!(
        gpu_authority.len(),
        1,
        "the actual GPU indirect authority must contain only the drawable entity's command"
    );
    assert_eq!(
        gpu_authority.iter().map(|row| row[1]).collect::<Vec<_>>(),
        vec![2],
        "GPU authority must identify the only drawable entity"
    );
    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.draw_ref_index)
            .collect::<Vec<_>>(),
        vec![0, 1],
        "both frame-owned snapshot segments must retain their draw-ref identities"
    );
    assert_eq!(
        gpu_args_in_submission_order(&words),
        vec![(6, 0)],
        "the GPU readback must separately report the one actual six-vertex draw"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_changes_when_fallback_slot_authority_changes() {
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_slot_raster");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSlotRaster",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_tiled_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("tiled_quad.obj"),
        8,
    );
    write_material(
        paths
            .asset_root(&RelPath::project_assets())
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
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![cluster(2, 20, 300, 0)],
        vec![page(300, false), page(400, true), page(800, true)],
    );
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let ((narrow_debug, narrow_rgba, narrow_words), (wide_debug, wide_rgba, wide_words)) = fixture
        .render_compiled_virtual_geometry_captured_test_frame_pair(
            viewport_size,
            (extract.clone(), Some(single_pending_prepare_frame(400))),
            (extract, Some(single_pending_prepare_frame(800))),
            &compiled,
        )
        .unwrap();
    let narrow_segments = narrow_debug
        .as_ref()
        .expect("first submitted frame exposes its execution segments")
        .execution_segments
        .as_slice();
    let wide_segments = wide_debug
        .as_ref()
        .expect("second submitted frame exposes its execution segments")
        .execution_segments
        .as_slice();
    let narrow_authority = gpu_authority_rows_in_args_order(&narrow_words);
    let wide_authority = gpu_authority_rows_in_args_order(&wide_words);
    let narrow_coverage = count_non_background_pixels(&narrow_rgba);
    let wide_coverage = count_non_background_pixels(&wide_rgba);

    assert_eq!(
        narrow_segments.iter().map(|segment| (
            segment.original_index,
            segment.cluster_span_count,
            segment.cluster_total_count,
            segment.page_id,
            segment.submission_slot,
            segment.state,
            segment.lineage_depth,
            segment.lod_level,
            segment.frontier_rank,
        )).collect::<Vec<_>>(),
        vec![(0, 1, 1, 300, Some(1), RenderVirtualGeometryExecutionState::PendingUpload, 0, 0, 0)],
        "expected pending submission to preserve the page-table-derived fallback slot authority even when the page is not resident yet"
    );
    assert_eq!(
        wide_segments.iter().map(|segment| (
            segment.original_index,
            segment.cluster_span_count,
            segment.cluster_total_count,
            segment.page_id,
            segment.submission_slot,
            segment.state,
            segment.lineage_depth,
            segment.lod_level,
            segment.frontier_rank,
        )).collect::<Vec<_>>(),
        vec![(0, 1, 1, 300, Some(7), RenderVirtualGeometryExecutionState::PendingUpload, 0, 0, 0)],
        "expected a different fallback recycle target to project a different authoritative submission slot into the GPU segment buffer"
    );
    assert_ne!(
        gpu_args_in_submission_order(&narrow_words),
        gpu_args_in_submission_order(&wide_words),
        "expected fallback recycle-slot authority to change actual GPU indirect args, not just uploader-side slot selection"
    );
    assert_eq!(narrow_authority.len(), 1);
    assert_eq!(wide_authority.len(), 1);
    assert_eq!(narrow_authority[0][3], 300);
    assert_eq!(wide_authority[0][3], 300);
    assert_eq!(narrow_authority[0][10], 1);
    assert_eq!(wide_authority[0][10], 7);
    assert_ne!(
        narrow_rgba, wide_rgba,
        "expected deeper cluster raster consumption to change when fallback slot authority changes, rather than stopping at prepare-time uploader hints"
    );
    assert!(
        narrow_coverage.abs_diff(wide_coverage) > 96,
        "expected fallback slot authority to materially change cluster raster coverage; narrow_coverage={narrow_coverage}, wide_coverage={wide_coverage}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_indirect_args_buffer_order_follows_fallback_slot_submission_authority() {
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_slot_indirect_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSlotIndirectOrder",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_tiled_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("tiled_quad.obj"),
        4,
    );
    write_material(
        paths
            .asset_root(&RelPath::project_assets())
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
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![cluster(2, 20, 300, 0), cluster(2, 30, 301, 1)],
        vec![
            page(300, false),
            page(301, false),
            page(400, true),
            page(800, true),
        ],
    );
    let compiled = compile_virtual_geometry_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let ((colder_debug, _, colder_words), (hotter_debug, _, hotter_words)) = fixture
        .render_compiled_virtual_geometry_captured_test_frame_pair(
            viewport_size,
            (extract.clone(), Some(dual_pending_prepare_frame(800, 400))),
            (extract, Some(dual_pending_prepare_frame(400, 800))),
            &compiled,
        )
        .unwrap();
    let colder_debug = colder_debug.expect("first submitted frame exposes its debug snapshot");
    let hotter_debug = hotter_debug.expect("second submitted frame exposes its debug snapshot");

    assert_eq!(
        gpu_args_in_submission_order(&colder_words),
        vec![(15, 3), (3, 3)],
        "expected indirect-args buffer order to follow the authoritative fallback slot submission order, so the colder slot-1 segment appears before the later slot-7 segment instead of preserving raw pending-draw insertion order"
    );
    assert_eq!(
        colder_debug
            .execution_segments
            .iter()
            .map(|segment| segment.page_id)
            .collect::<Vec<_>>(),
        vec![301, 300],
        "the debug snapshot retains the same fallback-slot order as the submitted GPU arguments"
    );
    assert_eq!(
        hotter_debug
            .execution_segments
            .iter()
            .map(|segment| segment.page_id)
            .collect::<Vec<_>>(),
        vec![300, 301],
        "swapping fallback ownership changes the frame snapshot order with the GPU arguments"
    );
    assert_eq!(
        gpu_authority_rows_in_args_order(&colder_words)
            .iter()
            .map(|row| (row[3], row[6]))
            .collect::<Vec<_>>(),
        vec![(301, 1), (300, 0)]
    );
    assert_eq!(
        gpu_authority_rows_in_args_order(&hotter_words)
            .iter()
            .map(|row| (row[3], row[6]))
            .collect::<Vec<_>>(),
        vec![(300, 0), (301, 1)]
    );
    assert_eq!(
        gpu_args_in_submission_order(&hotter_words),
        vec![(3, 6), (12, 3)],
        "expected swapping fallback slot authority to reshuffle the real indirect-args buffer order together with the segment truth, not just rewrite segment indices inside a fixed CPU draw order"
    );

    let _ = fs::remove_dir_all(root);
}

fn dual_pending_prepare_frame(
    page_300_recycled_page_id: u32,
    page_301_recycled_page_id: u32,
) -> VirtualGeometryPrepareFrame {
    VirtualGeometryPrepareFrame {
        visible_entities: vec![2],
        visible_clusters: vec![
            VirtualGeometryPrepareCluster {
                entity: 2,
                cluster_id: 20,
                page_id: 300,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            VirtualGeometryPrepareCluster {
                entity: 2,
                cluster_id: 30,
                page_id: 301,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        cluster_draw_segments: vec![
            draw_segment(2, 20, 300, 0, 2),
            draw_segment(2, 30, 301, 1, 2),
        ],
        resident_pages: vec![
            VirtualGeometryPreparePage {
                page_id: 400,
                slot: 1,
                size_bytes: 4096,
            },
            VirtualGeometryPreparePage {
                page_id: 800,
                slot: 7,
                size_bytes: 4096,
            },
        ],
        pending_page_requests: vec![
            VirtualGeometryPrepareRequest {
                page_id: 300,
                size_bytes: 4096,
                generation: 1,
                frontier_rank: 0,
                assigned_slot: None,
                recycled_page_id: Some(page_300_recycled_page_id),
            },
            VirtualGeometryPrepareRequest {
                page_id: 301,
                size_bytes: 4096,
                generation: 2,
                frontier_rank: 1,
                assigned_slot: None,
                recycled_page_id: Some(page_301_recycled_page_id),
            },
        ],
        available_slots: Vec::new(),
        evictable_pages: vec![
            VirtualGeometryPreparePage {
                page_id: 400,
                slot: 1,
                size_bytes: 4096,
            },
            VirtualGeometryPreparePage {
                page_id: 800,
                slot: 7,
                size_bytes: 4096,
            },
        ],
    }
}

fn single_pending_prepare_frame(recycled_page_id: u32) -> VirtualGeometryPrepareFrame {
    VirtualGeometryPrepareFrame {
        visible_entities: vec![2],
        visible_clusters: vec![VirtualGeometryPrepareCluster {
            entity: 2,
            cluster_id: 20,
            page_id: 300,
            lod_level: 0,
            resident_slot: None,
            state: VirtualGeometryPrepareClusterState::PendingUpload,
        }],
        cluster_draw_segments: vec![draw_segment(2, 20, 300, 0, 1)],
        resident_pages: vec![
            VirtualGeometryPreparePage {
                page_id: 400,
                slot: 1,
                size_bytes: 4096,
            },
            VirtualGeometryPreparePage {
                page_id: 800,
                slot: 7,
                size_bytes: 4096,
            },
        ],
        pending_page_requests: vec![VirtualGeometryPrepareRequest {
            page_id: 300,
            size_bytes: 4096,
            generation: 3,
            frontier_rank: 0,
            assigned_slot: None,
            recycled_page_id: Some(recycled_page_id),
        }],
        available_slots: Vec::new(),
        evictable_pages: vec![
            VirtualGeometryPreparePage {
                page_id: 400,
                slot: 1,
                size_bytes: 4096,
            },
            VirtualGeometryPreparePage {
                page_id: 800,
                slot: 7,
                size_bytes: 4096,
            },
        ],
    }
}

fn draw_segment(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    cluster_ordinal: u32,
    cluster_count: u32,
) -> VirtualGeometryPrepareDrawSegment {
    VirtualGeometryPrepareDrawSegment {
        entity,
        cluster_id,
        page_id,
        resident_slot: None,
        cluster_ordinal,
        cluster_span_count: 1,
        cluster_count,
        lineage_depth: 0,
        lod_level: 0,
        state: VirtualGeometryPrepareClusterState::PendingUpload,
    }
}

fn cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    _cluster_ordinal: u32,
) -> RenderVirtualGeometryCluster {
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

fn page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_graphics_{label}_{unique}"))
}

fn build_single_entity_extract_with_clusters(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
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
            meshes: vec![RenderMeshSnapshot {
                node_id: 2,
                stable_instance_key: 2 << 16,
                transform_revision: 0,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
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
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: clusters.len() as u32,
        page_budget: 2,
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

fn build_dual_entity_extract_with_clusters(
    viewport_size: UVec2,
    first_mesh: (u64, ResourceHandle<ModelMarker>),
    second_mesh: (u64, ResourceHandle<ModelMarker>),
    material: ResourceHandle<MaterialMarker>,
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
                    node_id: first_mesh.0,
                    stable_instance_key: first_mesh.0 << 16,
                    transform_revision: 0,
                    transform: Transform {
                        translation: Vec3::new(-0.45, 0.0, 0.0),
                        scale: Vec3::new(0.4, 0.4, 1.0),
                        ..Transform::default()
                    },
                    model: first_mesh.1,
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
                },
                RenderMeshSnapshot {
                    node_id: second_mesh.0,
                    stable_instance_key: second_mesh.0 << 16,
                    transform_revision: 0,
                    transform: Transform {
                        translation: Vec3::new(0.45, 0.0, 0.0),
                        scale: Vec3::new(0.4, 0.4, 1.0),
                        ..Transform::default()
                    },
                    model: second_mesh.1,
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
    extract.apply_viewport_size(viewport_size);
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

fn write_tiled_quad_obj(path: PathBuf, cluster_triangle_count: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    enable_fixture_virtual_geometry_cook(&path, cluster_triangle_count);
    fs::write(
        path,
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
}

fn write_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    enable_fixture_virtual_geometry_cook(&path, 2);
    fs::write(
        path,
        "v -1.0 -1.0 0.0\nv 1.0 -1.0 0.0\nv 1.0 1.0 0.0\nv -1.0 1.0 0.0\nvt 0.0 0.0\nvt 1.0 0.0\nvt 1.0 1.0\nvt 0.0 1.0\nvn 0.0 0.0 1.0\nf 1/1/1 2/2/1 3/3/1\nf 1/1/1 3/3/1 4/4/1\n",
    )
    .unwrap();
}

fn write_material(path: PathBuf, shader_uri: &str, texture_uri: &str) {
    write_material_with_color_and_alpha_mode(
        path,
        "VirtualGeometrySubmissionAuthority",
        shader_uri,
        texture_uri,
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );
}

fn write_material_with_color_and_alpha_mode(
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
        parent: None,
        options: Default::default(),
        queue: None,
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
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let project_root = fixture_project_root(&path);
    let document = material
        .to_project_toml_string(|reference| {
            Ok(persisted_reference_for_fixture(
                &project_root,
                &path,
                reference,
            ))
        })
        .unwrap();
    fs::write(path, document).unwrap();
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

fn count_non_background_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[0] > 4 || pixel[1] > 4 || pixel[2] > 4 || pixel[3] > 4)
        .count()
}

#[test]
fn virtual_geometry_transparent_submission_order_follows_visibility_owned_indirect_authority() {
    let root = unique_temp_project_root("graphics_virtual_geometry_submission_execution_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySubmissionExecutionOrder",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material_with_color_and_alpha_mode(
        paths
            .asset_root(&RelPath::project_assets())
            .join("materials")
            .join("red.zmaterial"),
        "VisibilityOrderRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material_with_color_and_alpha_mode(
        paths
            .asset_root(&RelPath::project_assets())
            .join("materials")
            .join("blue.zmaterial"),
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
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/red.zmaterial");
    let blue_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/blue.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, red_material), (3, blue_material)],
        vec![cluster(2, 20, 300, 0), cluster(3, 30, 301, 1)],
        vec![page(300, true), page(301, true)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let (_red_debug, red_rgba, red_words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract.clone(),
            Some(prepare_frame((2, 2), (3, 1))),
            &compiled,
        )
        .unwrap();
    let (_blue_debug, blue_rgba, blue_words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
            &compiled,
        )
        .unwrap();
    let red_dominant_center = center_pixel(&red_rgba, viewport_size);
    let blue_dominant_center = center_pixel(&blue_rgba, viewport_size);
    let red_dominant_segments = gpu_segment_order(&red_words);
    let blue_dominant_segments = gpu_segment_order(&blue_words);

    assert_eq!(
        red_dominant_segments,
        vec![(301, 3, 1, 0), (300, 2, 2, 0)],
        "expected the visibility-owned indirect segment order itself to swap when prepare-owned submission slots swap"
    );
    assert_eq!(
        blue_dominant_segments,
        vec![(300, 2, 1, 0), (301, 3, 2, 0)],
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
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSubmissionOrder",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material_with_color_and_alpha_mode(
        paths
            .asset_root(&RelPath::project_assets())
            .join("materials")
            .join("red.zmaterial"),
        "FallbackVisibilityOrderRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
        AlphaMode::Blend,
    );
    write_material_with_color_and_alpha_mode(
        paths
            .asset_root(&RelPath::project_assets())
            .join("materials")
            .join("blue.zmaterial"),
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
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/red.zmaterial");
    let blue_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/blue.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, red_material), (3, blue_material)],
        vec![cluster(2, 20, 300, 0), cluster(3, 30, 301, 1)],
        vec![page(300, true), page(301, true)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let (_red_debug, red_rgba, red_words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract.clone(),
            Some(prepare_frame_without_segments((2, 2), (3, 1))),
            &compiled,
        )
        .unwrap();
    let (_blue_debug, blue_rgba, blue_words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract,
            Some(prepare_frame_without_segments((2, 1), (3, 2))),
            &compiled,
        )
        .unwrap();
    let red_dominant_center = center_pixel(&red_rgba, viewport_size);
    let blue_dominant_center = center_pixel(&blue_rgba, viewport_size);
    let red_dominant_segments = gpu_segment_order(&red_words);
    let blue_dominant_segments = gpu_segment_order(&blue_words);

    assert_eq!(
        red_dominant_segments,
        vec![(301, 3, 1, 0), (300, 2, 2, 0)],
        "expected missing explicit draw segments to reuse the same visibility-owned unified indirect ordering, so synthesized fallback cluster draws still enter real submission in authoritative slot order"
    );
    assert_eq!(
        blue_dominant_segments,
        vec![(300, 2, 1, 0), (301, 3, 2, 0)],
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
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryGpuArgsSubmissionIndex",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material_with_color_and_alpha_mode(
        paths
            .asset_root(&RelPath::project_assets())
            .join("materials")
            .join("white.zmaterial"),
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
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, material.clone()), (3, material)],
        vec![cluster(2, 20, 300, 0), cluster(3, 30, 301, 1)],
        vec![page(300, true), page(301, true)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let (_debug, _rgba, words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract,
            Some(prepare_frame((2, 2), (3, 1))),
            &compiled,
        )
        .unwrap();
    let indirect_args = gpu_first_instance_words(&words);

    assert_eq!(
        indirect_args,
        vec![0, 1 << 16],
        "expected the real GPU-generated indirect args source to encode visibility-owned submission tokens into first_instance so unified-indirect authority reaches the actual args buffer instead of staying on the parallel debug token side-channel"
    );
    let rows = gpu_authority_rows(&words);
    let authority_tokens = rows
        .iter()
        .map(|row| (row[8] << 16) | row[9])
        .collect::<Vec<_>>();
    assert_eq!(
        authority_tokens, indirect_args,
        "expected GPU first_instance tokens to correspond to the same GPU authority records"
    );
    assert_eq!(
        rows.iter().map(|row| row[0]).collect::<Vec<_>>(),
        vec![0, 1],
        "expected authority records to retain actual per-instance indices independently of visibility tokens"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_renderer_mesh_draw_submission_order_tracks_visibility_owned_unified_indirect_authority(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_renderer_submission_order_truth");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryRendererSubmissionOrderTruth",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("flat_color.wgsl"),
        [1.0, 1.0, 1.0],
    );
    write_solid_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material_with_color_and_alpha_mode(
        paths
            .asset_root(&RelPath::project_assets())
            .join("materials")
            .join("white.zmaterial"),
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
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_overlapping_extract(
        viewport_size,
        model,
        [(2, material.clone()), (3, material)],
        vec![cluster(2, 20, 300, 0), cluster(3, 30, 301, 1)],
        vec![page(300, true), page(301, true)],
    );

    let compiled = compile_virtual_geometry_forward_plus_pipeline(&extract);

    let mut fixture = pluginized_wgpu_render_framework_with_gpu_test_collector(asset_manager);
    let (_red_debug, _red_rgba, red_words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract.clone(),
            Some(prepare_frame((2, 2), (3, 1))),
            &compiled,
        )
        .unwrap();
    let red_dominant_order = gpu_mesh_submission_order(&red_words);
    let (_blue_debug, _blue_rgba, blue_words) = fixture
        .render_compiled_virtual_geometry_captured_test_frame(
            viewport_size,
            extract,
            Some(prepare_frame((2, 1), (3, 2))),
            &compiled,
        )
        .unwrap();
    let blue_dominant_order = gpu_mesh_submission_order(&blue_words);

    assert_eq!(
        red_dominant_order,
        vec![(1, 3, 301, 1, 0), (0, 2, 300, 2, 0)],
        "expected renderer-side mesh draw submission order to preserve per-instance ownership alongside the same visibility-owned unified indirect order when entity 3 owns the earlier authoritative slot"
    );
    assert_eq!(
        blue_dominant_order,
        vec![(0, 2, 300, 1, 0), (1, 3, 301, 2, 0)],
        "expected renderer-side mesh draw submission order to flip with the same visibility-owned unified indirect authority while keeping the matching instance ownership instead of staying on fixed CPU mesh insertion order"
    );

    let _ = fs::remove_dir_all(root);
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
    let instances = snapshot
        .scene
        .meshes
        .iter()
        .map(|mesh| {
            let cluster_rows = clusters
                .iter()
                .enumerate()
                .filter(|(_, row)| row.entity == mesh.node_id)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            assert_eq!(cluster_rows.len(), 1, "fixture mesh owns one cluster row");
            let cluster_offset = cluster_rows[0];
            let page_rows = pages
                .iter()
                .enumerate()
                .filter(|(_, row)| row.page_id == clusters[cluster_offset].page_id)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            assert_eq!(page_rows.len(), 1, "fixture cluster owns one page row");
            zircon_runtime::core::framework::render::RenderVirtualGeometryInstance {
                entity: mesh.node_id,
                stable_instance_key: mesh.stable_instance_key,
                source_model: Some(mesh.model.id()),
                transform: mesh.transform,
                cluster_offset: u32::try_from(cluster_offset).expect("fixture cluster offset"),
                cluster_count: 1,
                page_offset: u32::try_from(page_rows[0]).expect("fixture page offset"),
                page_count: 1,
                ..Default::default()
            }
        })
        .collect();
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: clusters.len() as u32,
        page_budget: pages.len() as u32,
        clusters,
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages,
        page_dependencies: Vec::new(),
        instances,
        debug: Default::default(),
    });
    extract
}

fn prepare_frame(first: (u64, u32), second: (u64, u32)) -> VirtualGeometryPrepareFrame {
    VirtualGeometryPrepareFrame {
        visible_entities: vec![first.0, second.0],
        visible_clusters: vec![
            VirtualGeometryPrepareCluster {
                entity: first.0,
                cluster_id: 20,
                page_id: 300,
                lod_level: 0,
                resident_slot: Some(first.1),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareCluster {
                entity: second.0,
                cluster_id: 30,
                page_id: 301,
                lod_level: 0,
                resident_slot: Some(second.1),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        cluster_draw_segments: vec![
            resident_draw_segment(first.0, 20, 300, first.1),
            resident_draw_segment(second.0, 30, 301, second.1),
        ],
        resident_pages: vec![
            VirtualGeometryPreparePage {
                page_id: 300,
                slot: first.1,
                size_bytes: 4096,
            },
            VirtualGeometryPreparePage {
                page_id: 301,
                slot: second.1,
                size_bytes: 4096,
            },
        ],
        pending_page_requests: Vec::new(),
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    }
}

fn prepare_frame_without_segments(
    first: (u64, u32),
    second: (u64, u32),
) -> VirtualGeometryPrepareFrame {
    let mut prepare = prepare_frame(first, second);
    prepare.cluster_draw_segments.clear();
    prepare
}

fn resident_draw_segment(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    slot: u32,
) -> VirtualGeometryPrepareDrawSegment {
    VirtualGeometryPrepareDrawSegment {
        entity,
        cluster_id,
        page_id,
        resident_slot: Some(slot),
        cluster_ordinal: 0,
        cluster_span_count: 1,
        cluster_count: 1,
        lineage_depth: 0,
        lod_level: 0,
        state: VirtualGeometryPrepareClusterState::Resident,
    }
}

fn gpu_authority_rows(
    words: &crate::render_pass_executors::IndirectReadbackWords,
) -> Vec<[u32; 14]> {
    words
        .authority
        .chunks_exact(14)
        .filter(|row| row[3] != 0)
        .map(|row| row.try_into().unwrap())
        .collect()
}

fn gpu_first_instance_words(
    words: &crate::render_pass_executors::IndirectReadbackWords,
) -> Vec<u32> {
    let count = words.args.first().copied().unwrap_or_default() as usize;
    words
        .args
        .get(1..1 + count.saturating_mul(4))
        .unwrap_or_default()
        .chunks_exact(4)
        .filter(|args| args[1] != 0)
        .map(|args| args[3])
        .collect()
}

fn gpu_segment_order(
    words: &crate::render_pass_executors::IndirectReadbackWords,
) -> Vec<(u32, u32, u32, u32)> {
    let tokens = gpu_first_instance_words(words);
    let mut rows = gpu_authority_rows(words)
        .into_iter()
        .filter(|row| tokens.contains(&((row[8] << 16) | row[9])))
        .map(|row| (row[3], row[1], row[8], row[9]))
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| (row.2, row.3));
    rows
}

fn gpu_mesh_submission_order(
    words: &crate::render_pass_executors::IndirectReadbackWords,
) -> Vec<(u32, u32, u32, u32, u32)> {
    let tokens = gpu_first_instance_words(words);
    let mut rows = gpu_authority_rows(words)
        .into_iter()
        .filter(|row| tokens.contains(&((row[8] << 16) | row[9])))
        .map(|row| (row[0], row[1], row[3], row[8], row[9]))
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| (row.3, row.4));
    rows
}

fn compile_virtual_geometry_forward_plus_pipeline(
    extract: &RenderFrameExtract,
) -> CompiledRenderPipeline {
    compile_virtual_geometry_pipeline(extract)
}
fn center_pixel(rgba: &[u8], viewport_size: UVec2) -> [u8; 4] {
    let width = usize::try_from(viewport_size.x).expect("viewport width fits usize");
    let height = usize::try_from(viewport_size.y).expect("viewport height fits usize");
    let offset = (height / 2)
        .checked_mul(width)
        .and_then(|row| row.checked_add(width / 2))
        .and_then(|pixel| pixel.checked_mul(4))
        .expect("center pixel offset fits usize");
    rgba.get(offset..offset.checked_add(4).expect("pixel end fits usize"))
        .expect("actual capture contains center pixel")
        .try_into()
        .expect("RGBA has four bytes")
}

fn fixture_project_root(source_path: &Path) -> PathBuf {
    source_path
        .ancestors()
        .find(|candidate| candidate.join("zircon-project.toml").is_file())
        .expect("asset fixture must live below a project manifest")
        .to_path_buf()
}

// 夹具只把 res:// 路径映射到唯一资源根的持久化 hint；多根歧义必须显式失败。
fn persisted_reference_for_fixture(
    project_root: &Path,
    source_path: &Path,
    reference: &AssetReference,
) -> PersistedAssetReference {
    if reference.locator.scheme() == ResourceScheme::Builtin {
        return PersistedAssetReference::builtin(reference.locator.clone());
    }
    assert_eq!(reference.locator.scheme(), ResourceScheme::Res);

    let manifest = ProjectManifest::load(project_root.join("zircon-project.toml")).unwrap();
    let roots = manifest
        .asset_roots
        .iter()
        .filter(|root| {
            root.join_to(project_root)
                .join(reference.locator.path())
                .is_file()
        })
        .collect::<Vec<_>>();
    let root = match roots.as_slice() {
        [root] => *root,
        [] => manifest
            .asset_roots
            .iter()
            .find(|root| source_path.starts_with(root.join_to(project_root)))
            .expect("asset fixture source must live below a manifest asset root"),
        _ => panic!(
            "asset fixture reference {} resolves through multiple project asset roots",
            reference.locator
        ),
    };
    let path_hint =
        RelPath::parse(format!("{}/{}", root.as_str(), reference.locator.path())).unwrap();
    PersistedAssetReference::project(
        AssetRef::try_new(
            reference.uuid,
            path_hint,
            reference.locator.label().map(str::to_owned),
        )
        .unwrap(),
    )
}

fn enable_fixture_virtual_geometry_cook(path: &Path, cluster_triangle_count: usize) {
    let file_name = path.file_name().unwrap();
    let uri = format!("res://models/{}", file_name.to_str().unwrap());
    let mut metadata = zircon_runtime::asset::project::AssetMetaDocument::new(
        zircon_runtime::asset::AssetUuid::new(),
        AssetUri::parse(&uri).unwrap(),
        zircon_runtime::asset::AssetKind::Model,
    );
    metadata.import_settings = format!(
        "[virtual_geometry]\nenabled = true\ncluster_triangle_count = {cluster_triangle_count}\npage_cluster_count = 1\n"
    )
    .parse()
    .unwrap();
    let mut sidecar_name = file_name.to_os_string();
    sidecar_name.push(".zmeta");
    metadata.save(path.with_file_name(sidecar_name)).unwrap();
}
