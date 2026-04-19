use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_asset::assets::{AlphaMode, MaterialAsset};
use zircon_asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_asset::{AssetReference, AssetUri};
use zircon_framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use zircon_math::{Transform, UVec2, Vec3, Vec4};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_scene::components::{default_render_layer_mask, Mobility};

use crate::{
    types::{
        EditorOrRuntimeFrame, VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
        VirtualGeometryPrepareRequest,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

#[test]
fn virtual_geometry_unified_indirect_uses_fallback_recycle_slot_authority_for_submission_order_and_draw_refs(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_slot_submission");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSlotSubmission",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_tiled_quad_obj(paths.assets_root().join("models").join("tiled_quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.material.toml"),
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
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
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
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
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
        .unwrap();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(dual_pending_prepare_frame(800, 400))),
            &compiled,
            None,
        )
        .unwrap();
    let colder_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let colder_draw_ref_indices = renderer
        .read_last_virtual_geometry_indirect_draw_refs()
        .unwrap()
        .into_iter()
        .map(|(_mesh_index_count, segment_index)| segment_index)
        .collect::<Vec<_>>();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(dual_pending_prepare_frame(400, 800))),
            &compiled,
            None,
        )
        .unwrap();
    let hotter_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let hotter_draw_ref_indices = renderer
        .read_last_virtual_geometry_indirect_draw_refs()
        .unwrap()
        .into_iter()
        .map(|(_mesh_index_count, segment_index)| segment_index)
        .collect::<Vec<_>>();

    assert_eq!(
        colder_segments,
        vec![
            (
                1,
                1,
                2,
                301,
                1,
                VirtualGeometryPrepareClusterState::PendingUpload,
                0,
                0,
                1,
            ),
            (
                0,
                1,
                2,
                300,
                7,
                VirtualGeometryPrepareClusterState::PendingUpload,
                0,
                0,
                0,
            ),
        ],
        "expected fallback recycle-slot truth to continue into the real GPU-submitted segment ordering so unified indirect authority follows the colder page-table slot even before upload completion"
    );
    assert_eq!(
        colder_draw_ref_indices,
        vec![0, 1],
        "expected draw-ref buffer order itself to follow the authoritative fallback slot submission order so the colder slot-owned segment is encoded first instead of only remapping fixed CPU draw order"
    );
    assert_eq!(
        hotter_segments,
        vec![
            (
                0,
                1,
                2,
                300,
                1,
                VirtualGeometryPrepareClusterState::PendingUpload,
                0,
                0,
                0,
            ),
            (
                1,
                1,
                2,
                301,
                7,
                VirtualGeometryPrepareClusterState::PendingUpload,
                0,
                0,
                1,
            ),
        ],
        "expected swapping fallback recycle preference to swap the authoritative submission-slot order instead of leaving unified indirect ownership stuck on raw segment insertion order"
    );
    assert_eq!(
        hotter_draw_ref_indices,
        vec![0, 1],
        "expected draw-ref buffer order to stay aligned with the authoritative slot-sorted segment order after the hotter slot-owner swap, rather than preserving pre-sort CPU insertion semantics"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_segment_buffer_keeps_prepare_owned_segments_when_some_entities_do_not_emit_pending_draws(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_prepare_owned_segments");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPrepareOwnedSegments",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
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
            .join("flat_green.material.toml"),
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
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, valid_model),
        (3, valid_model),
        green_material,
        vec![cluster(2, 20, 300, 0), cluster(3, 30, 301, 0)],
        vec![page(300, true), page(301, true)],
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
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
        .unwrap();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
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
                        crate::types::VirtualGeometryPrepareDrawSegment {
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
                        crate::types::VirtualGeometryPrepareDrawSegment {
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
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        renderer.last_virtual_geometry_indirect_segment_count(),
        2,
        "expected the shared indirect segment buffer to keep both prepare-owned visibility segments even when only one entity can emit a pending mesh draw"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_segments().unwrap(),
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
        "expected segment authority to come from prepare-owned visibility truth instead of only the subset of entities that happened to build renderer pending draws"
    );
    assert_eq!(
        renderer.last_virtual_geometry_indirect_draw_count(),
        1,
        "expected only the drawable entity to submit a mesh draw even while shared args source authority continues moving up to prepare-owned visibility truth"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_draw_refs().unwrap(),
        vec![(6, 0), (6, 1)],
        "expected the shared draw-ref buffer to retain one prepare-owned record per visibility-owned segment, even though only one entity still emitted an actual mesh draw this frame"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_changes_when_fallback_slot_authority_changes() {
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_slot_raster");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSlotRaster",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_tiled_quad_obj(paths.assets_root().join("models").join("tiled_quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.material.toml"),
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
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![cluster(2, 20, 300, 0)],
        vec![page(300, false), page(400, true), page(800, true)],
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
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
        .unwrap();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let narrow_slot = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(single_pending_prepare_frame(400))),
            &compiled,
            None,
        )
        .unwrap();
    let narrow_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();
    let narrow_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();

    let wide_slot = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(single_pending_prepare_frame(800))),
            &compiled,
            None,
        )
        .unwrap();
    let wide_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();
    let wide_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();

    let narrow_coverage = count_non_background_pixels(&narrow_slot.rgba);
    let wide_coverage = count_non_background_pixels(&wide_slot.rgba);

    assert_eq!(
        narrow_segments,
        vec![(
            0,
            1,
            1,
            300,
            1,
            VirtualGeometryPrepareClusterState::PendingUpload,
            0,
            0,
            0,
        )],
        "expected pending submission to preserve the page-table-derived fallback slot authority even when the page is not resident yet"
    );
    assert_eq!(
        wide_segments,
        vec![(
            0,
            1,
            1,
            300,
            7,
            VirtualGeometryPrepareClusterState::PendingUpload,
            0,
            0,
            0,
        )],
        "expected a different fallback recycle target to project a different authoritative submission slot into the GPU segment buffer"
    );
    assert_ne!(
        narrow_args, wide_args,
        "expected fallback recycle-slot authority to change the real GPU indirect args, not just uploader-side slot selection"
    );
    assert_ne!(
        narrow_slot.rgba, wide_slot.rgba,
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
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackSlotIndirectOrder",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_green.wgsl"),
        [0.08, 0.95, 0.08],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("white.png"),
        [255, 255, 255, 255],
    );
    write_tiled_quad_obj(paths.assets_root().join("models").join("tiled_quad.obj"));
    write_material(
        paths
            .assets_root()
            .join("materials")
            .join("flat_green.material.toml"),
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
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
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
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
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
        .unwrap();

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(dual_pending_prepare_frame(800, 400))),
            &compiled,
            None,
        )
        .unwrap();
    let colder_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(dual_pending_prepare_frame(400, 800))),
            &compiled,
            None,
        )
        .unwrap();
    let hotter_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    assert_eq!(
        colder_args,
        vec![(15, 3), (3, 3)],
        "expected indirect-args buffer order to follow the authoritative fallback slot submission order, so the colder slot-1 segment appears before the later slot-7 segment instead of preserving raw pending-draw insertion order"
    );
    assert_eq!(
        hotter_args,
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
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                model,
                material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            }],
            lights: Vec::new(),
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
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: clusters.len() as u32,
        page_budget: 2,
        clusters,
        pages,
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
                    transform: Transform {
                        translation: Vec3::new(-0.45, 0.0, 0.0),
                        scale: Vec3::new(0.4, 0.4, 1.0),
                        ..Transform::default()
                    },
                    model: first_mesh.1,
                    material,
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
                RenderMeshSnapshot {
                    node_id: second_mesh.0,
                    transform: Transform {
                        translation: Vec3::new(0.45, 0.0, 0.0),
                        scale: Vec3::new(0.4, 0.4, 1.0),
                        ..Transform::default()
                    },
                    model: second_mesh.1,
                    material,
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
            ],
            lights: Vec::new(),
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
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: clusters.len() as u32,
        page_budget: pages.len() as u32,
        clusters,
        pages,
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

fn write_tiled_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
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
    fs::write(
        path,
        "v -1.0 -1.0 0.0\nv 1.0 -1.0 0.0\nv 1.0 1.0 0.0\nv -1.0 1.0 0.0\nvt 0.0 0.0\nvt 1.0 0.0\nvt 1.0 1.0\nvt 0.0 1.0\nvn 0.0 0.0 1.0\nf 1/1/1 2/2/1 3/3/1\nf 1/1/1 3/3/1 4/4/1\n",
    )
    .unwrap();
}

fn write_material(path: PathBuf, shader_uri: &str, texture_uri: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("VirtualGeometrySubmissionAuthority".to_string()),
        shader: asset_reference(shader_uri),
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

fn count_non_background_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[0] > 4 || pixel[1] > 4 || pixel[2] > 4 || pixel[3] > 4)
        .count()
}
