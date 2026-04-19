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
        VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

fn draw_segment(
    entity: u64,
    cluster_id: u32,
    cluster_ordinal: u32,
    cluster_count: u32,
    lod_level: u8,
    state: VirtualGeometryPrepareClusterState,
) -> crate::types::VirtualGeometryPrepareDrawSegment {
    crate::types::VirtualGeometryPrepareDrawSegment {
        entity,
        cluster_id,
        page_id: 0,
        resident_slot: None,
        cluster_ordinal,
        cluster_span_count: 1,
        cluster_count,
        lineage_depth: 0,
        lod_level,
        state,
    }
}

fn draw_segment_with_span(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    resident_slot: Option<u32>,
    cluster_ordinal: u32,
    cluster_span_count: u32,
    cluster_count: u32,
    lod_level: u8,
    state: VirtualGeometryPrepareClusterState,
) -> crate::types::VirtualGeometryPrepareDrawSegment {
    crate::types::VirtualGeometryPrepareDrawSegment {
        entity,
        cluster_id,
        page_id,
        resident_slot,
        cluster_ordinal,
        cluster_span_count,
        cluster_count,
        lineage_depth: u32::from(lod_level),
        lod_level,
        state,
    }
}

#[test]
fn virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities() {
    let root = unique_temp_project_root("graphics_virtual_geometry_prepare");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPrepare",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_red.wgsl"),
        [0.95, 0.08, 0.08],
    );
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
            .join("flat_red.material.toml"),
        "res://shaders/flat_red.wgsl",
        "res://textures/white.png",
    );
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
    let red_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_red.material.toml");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_extract(viewport_size, model, red_material, green_material);
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
    let unfiltered = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size),
            &compiled,
            None,
        )
        .unwrap();
    let filtered = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let unfiltered_green = average_channel(&unfiltered.rgba, 1);
    let filtered_red = average_channel(&filtered.rgba, 0);
    let filtered_green = average_channel(&filtered.rgba, 1);

    assert!(
        filtered_green > filtered_red + 8.0,
        "expected filtered Virtual Geometry fallback to keep only the green entity; red={filtered_red:.2}, green={filtered_green:.2}"
    );
    assert!(
        filtered_green > unfiltered_green + 4.0,
        "expected filtering to increase the green contribution; unfiltered={unfiltered_green:.2}, filtered={filtered_green:.2}"
    );
    assert_ne!(
        unfiltered.rgba, filtered.rgba,
        "expected Virtual Geometry prepare filtering to change the fallback frame output"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_streaming_state_changes_fallback_raster_output() {
    let root = unique_temp_project_root("graphics_virtual_geometry_streaming");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryStreaming",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_red.wgsl"),
        [0.95, 0.08, 0.08],
    );
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
            .join("flat_red.material.toml"),
        "res://shaders/flat_red.wgsl",
        "res://textures/white.png",
    );
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
    let red_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_red.material.toml");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_extract(viewport_size, model, red_material, green_material);
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
    let pending = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: None,
                        state: VirtualGeometryPrepareClusterState::PendingUpload,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::PendingUpload,
                    )],
                    resident_pages: Vec::new(),
                    pending_page_requests: vec![crate::types::VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 5,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let resident = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let pending_green = average_channel(&pending.rgba, 1);
    let resident_green = average_channel(&resident.rgba, 1);
    assert!(
        resident_green > pending_green + 6.0,
        "expected resident Virtual Geometry clusters to raster more strongly than pending clusters; pending={pending_green:.2}, resident={resident_green:.2}"
    );
    assert_ne!(
        pending.rgba, resident.rgba,
        "expected cluster streaming state to change fallback raster output"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_streaming_state_changes_fallback_raster_coverage() {
    let root = unique_temp_project_root("graphics_virtual_geometry_streaming_coverage");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryStreamingCoverage",
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
    let extract = build_single_entity_extract(viewport_size, model, green_material);
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
    let pending = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: None,
                        state: VirtualGeometryPrepareClusterState::PendingUpload,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::PendingUpload,
                    )],
                    resident_pages: Vec::new(),
                    pending_page_requests: vec![crate::types::VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 7,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let resident = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let pending_coverage = count_non_background_pixels(&pending.rgba);
    let resident_coverage = count_non_background_pixels(&resident.rgba);
    assert!(
        resident_coverage > pending_coverage + 150,
        "expected resident Virtual Geometry clusters to cover materially more pixels than pending clusters; pending={pending_coverage}, resident={resident_coverage}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_gpu_args_change_when_only_visible_submission_index_changes() {
    let root = unique_temp_project_root("graphics_virtual_geometry_submission_index_args");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySubmissionIndexArgs",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths.assets_root().join("shaders").join("flat_red.wgsl"),
        [0.95, 0.08, 0.08],
    );
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
            .join("flat_red.material.toml"),
        "res://shaders/flat_red.wgsl",
        "res://textures/white.png",
    );
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
    let red_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_red.material.toml");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_extract(viewport_size, model, red_material, green_material);
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
    let helper_precedes_frame = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 1,
                            cluster_id: 1,
                            page_id: 200,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(2),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![
                        draw_segment_with_span(
                            1,
                            1,
                            200,
                            Some(1),
                            0,
                            1,
                            1,
                            0,
                            VirtualGeometryPrepareClusterState::Resident,
                        ),
                        draw_segment_with_span(
                            2,
                            2,
                            300,
                            Some(2),
                            0,
                            1,
                            4,
                            0,
                            VirtualGeometryPrepareClusterState::Resident,
                        ),
                    ],
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 200,
                            slot: 1,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 300,
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
    let helper_precedes_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let helper_precedes_draw_refs = renderer
        .read_last_virtual_geometry_indirect_draw_refs()
        .unwrap();
    let helper_precedes_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();
    let helper_precedes_target_segment =
        indirect_segment_for_page(&helper_precedes_segments, 300).unwrap();
    let helper_precedes_target_args = indirect_args_for_page(
        &helper_precedes_segments,
        &helper_precedes_draw_refs,
        &helper_precedes_args,
        300,
    )
    .unwrap();

    let helper_follows_frame = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 1,
                            cluster_id: 1,
                            page_id: 200,
                            lod_level: 0,
                            resident_slot: Some(7),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(2),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![
                        draw_segment_with_span(
                            1,
                            1,
                            200,
                            Some(7),
                            0,
                            1,
                            1,
                            0,
                            VirtualGeometryPrepareClusterState::Resident,
                        ),
                        draw_segment_with_span(
                            2,
                            2,
                            300,
                            Some(2),
                            0,
                            1,
                            4,
                            0,
                            VirtualGeometryPrepareClusterState::Resident,
                        ),
                    ],
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 200,
                            slot: 7,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 300,
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
    let helper_follows_segments = renderer
        .read_last_virtual_geometry_indirect_segments()
        .unwrap();
    let helper_follows_draw_refs = renderer
        .read_last_virtual_geometry_indirect_draw_refs()
        .unwrap();
    let helper_follows_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();
    let helper_follows_target_segment =
        indirect_segment_for_page(&helper_follows_segments, 300).unwrap();
    let helper_follows_target_args = indirect_args_for_page(
        &helper_follows_segments,
        &helper_follows_draw_refs,
        &helper_follows_args,
        300,
    )
    .unwrap();

    assert_eq!(
        helper_precedes_target_segment,
        helper_follows_target_segment,
        "expected the visible entity's page/slot/state/frontier/lod/lineage truth to stay fixed while only authoritative submission order changes around it"
    );
    assert_ne!(
        helper_precedes_target_args,
        helper_follows_target_args,
        "expected visibility-owned submission order to change the real GPU-generated indirect args for the visible entity even when its own page/slot/state/frontier/lod/lineage stay fixed"
    );
    assert_ne!(
        helper_precedes_frame.rgba,
        helper_follows_frame.rgba,
        "expected the visible entity's real cluster-raster output to change once its authoritative submission index shifts, instead of leaving submission ownership at CPU ordering only"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_resident_slot_changes_fallback_raster_output() {
    let root = unique_temp_project_root("graphics_virtual_geometry_slot_indirection");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySlotIndirection",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract(viewport_size, model, green_material);
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
    let slot_one = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let slot_seven = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let slot_one_green = average_channel(&slot_one.rgba, 1);
    let slot_seven_green = average_channel(&slot_seven.rgba, 1);
    let slot_one_coverage = count_non_background_pixels(&slot_one.rgba);
    let slot_seven_coverage = count_non_background_pixels(&slot_seven.rgba);
    assert!(
        (slot_one_green - slot_seven_green).abs() > 2.0
            || slot_one_coverage.abs_diff(slot_seven_coverage) > 120,
        "expected resident slot indirection to affect Virtual Geometry fallback raster consumption; slot1_green={slot_one_green:.2}, slot7_green={slot_seven_green:.2}, slot1_coverage={slot_one_coverage}, slot7_coverage={slot_seven_coverage}"
    );
    assert_ne!(
        slot_one.rgba, slot_seven.rgba,
        "expected different resident slots to produce different fallback raster output"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_visible_cluster_ids_change_fallback_raster_region() {
    let root = unique_temp_project_root("graphics_virtual_geometry_cluster_region");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryClusterRegion",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_clustered_extract(viewport_size, model, green_material);
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
    let first_cluster = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        2,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let second_cluster = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 3,
                        page_id: 301,
                        lod_level: 1,
                        resident_slot: Some(2),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        3,
                        1,
                        2,
                        1,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 301,
                        slot: 2,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let first_left_green = average_half_channel(&first_cluster.rgba, viewport_size, 1, Half::Left);
    let first_right_green =
        average_half_channel(&first_cluster.rgba, viewport_size, 1, Half::Right);
    let second_left_green =
        average_half_channel(&second_cluster.rgba, viewport_size, 1, Half::Left);
    let second_right_green =
        average_half_channel(&second_cluster.rgba, viewport_size, 1, Half::Right);

    assert!(
        first_right_green > first_left_green + 8.0,
        "expected the first resident cluster to bias fallback raster coverage toward the right half; left={first_left_green:.2}, right={first_right_green:.2}"
    );
    assert!(
        second_left_green > second_right_green + 8.0,
        "expected the second resident cluster to bias fallback raster coverage toward the left half; left={second_left_green:.2}, right={second_right_green:.2}"
    );
    assert_ne!(
        first_cluster.rgba, second_cluster.rgba,
        "expected different visible resident cluster ids to produce different fallback raster regions"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_draw_segments_override_extract_cluster_ordinals_for_fallback_raster() {
    let root = unique_temp_project_root("graphics_virtual_geometry_draw_segments");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryDrawSegments",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_clustered_extract(viewport_size, model, green_material);
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
    let frame = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        1,
                        2,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let left_green = average_half_channel(&frame.rgba, viewport_size, 1, Half::Left);
    let right_green = average_half_channel(&frame.rgba, viewport_size, 1, Half::Right);
    assert!(
        left_green > right_green + 8.0,
        "expected explicit prepare draw segments to override extract-derived cluster ordinals and bias raster coverage toward the left half; left={left_green:.2}, right={right_green:.2}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_segments_submit_indirect_raster_draws_when_feature_enabled() {
    let root = unique_temp_project_root("graphics_virtual_geometry_indirect_raster");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryIndirectRaster",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_clustered_extract(viewport_size, model, green_material);
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
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment(
                        2,
                        2,
                        0,
                        2,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert!(
        renderer.last_virtual_geometry_indirect_draw_count() >= 1,
        "expected Virtual Geometry prepare segments to submit at least one indirect raster draw"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_preserves_explicit_segment_boundaries_even_when_segments_share_page_slot(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_slot_compaction");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySlotCompaction",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 3,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 0.8,
            },
        ],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
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
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![
                        draw_segment(2, 2, 0, 2, 0, VirtualGeometryPrepareClusterState::Resident),
                        draw_segment(2, 3, 1, 2, 0, VirtualGeometryPrepareClusterState::Resident),
                    ],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let same_slot_draw_count = renderer.last_virtual_geometry_indirect_draw_count();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(7),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![
                        draw_segment(2, 2, 0, 2, 0, VirtualGeometryPrepareClusterState::Resident),
                        draw_segment(2, 3, 1, 2, 0, VirtualGeometryPrepareClusterState::Resident),
                    ],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let different_slot_draw_count = renderer.last_virtual_geometry_indirect_draw_count();

    assert_eq!(
        same_slot_draw_count, 2,
        "expected renderer submission to keep the explicit prepare draw-segment boundaries even when adjacent resident segments share one page slot"
    );
    assert_eq!(
        different_slot_draw_count, 2,
        "expected explicit prepare draw-segment boundaries to stay split when the segments also resolve to different resident slots"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_reuses_one_shared_indirect_buffer_across_multiple_indirect_draws() {
    let root = unique_temp_project_root("graphics_virtual_geometry_shared_indirect_buffer");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySharedIndirectBuffer",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 3,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 0.8,
            },
        ],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
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
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(7),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![
                        draw_segment(2, 2, 0, 2, 0, VirtualGeometryPrepareClusterState::Resident),
                        draw_segment(2, 3, 1, 2, 0, VirtualGeometryPrepareClusterState::Resident),
                    ],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        renderer.last_virtual_geometry_indirect_draw_count(),
        2,
        "expected the split-slot prepare snapshot to keep two indirect draws"
    );
    assert_eq!(
        renderer.last_virtual_geometry_indirect_buffer_count(),
        1,
        "expected multiple Virtual Geometry indirect draws to reuse one shared indirect args buffer"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_keeps_different_pages_in_same_slot_split_into_separate_indirect_draws()
{
    let root = unique_temp_project_root("graphics_virtual_geometry_page_split_indirect_draws");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPageSplitIndirectDraws",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 3,
                page_id: 301,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 0.8,
            },
        ],
        vec![
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: true,
                size_bytes: 4096,
            },
            RenderVirtualGeometryPage {
                page_id: 301,
                resident: true,
                size_bytes: 4096,
            },
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 301,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![
                        draw_segment_with_span(
                            2,
                            2,
                            300,
                            Some(1),
                            0,
                            1,
                            2,
                            0,
                            VirtualGeometryPrepareClusterState::Resident,
                        ),
                        draw_segment_with_span(
                            2,
                            3,
                            301,
                            Some(1),
                            1,
                            1,
                            2,
                            0,
                            VirtualGeometryPrepareClusterState::Resident,
                        ),
                    ],
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 300,
                            slot: 1,
                            size_bytes: 4096,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 301,
                            slot: 1,
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
        renderer.last_virtual_geometry_indirect_draw_count(),
        2,
        "expected different resident pages to keep separate indirect draws even when they temporarily share one slot"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_visibility_owned_draw_segment_span_changes_fallback_coverage() {
    let root = unique_temp_project_root("graphics_virtual_geometry_visibility_owned_segments");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryVisibilityOwnedSegments",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 3,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 0.8,
            },
        ],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
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
    let narrow = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(7),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(1),
                        0,
                        1,
                        2,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let narrow_draw_count = renderer.last_virtual_geometry_indirect_draw_count();
    let wide = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(7),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(1),
                        0,
                        2,
                        2,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let wide_draw_count = renderer.last_virtual_geometry_indirect_draw_count();

    let narrow_green = dominant_green_pixels(&narrow.rgba);
    let wide_green = dominant_green_pixels(&wide.rgba);
    assert_eq!(
        narrow_draw_count, 1,
        "expected one visibility-owned indirect draw for the narrow segment"
    );
    assert_eq!(
        wide_draw_count, 1,
        "expected one visibility-owned indirect draw for the precompacted wide segment"
    );
    assert!(
        wide_green > narrow_green + 64,
        "expected visibility-owned draw-segment span to increase fallback coverage; narrow={narrow_green}, wide={wide_green}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_follow_visibility_owned_segment_span() {
    let root = unique_temp_project_root("graphics_virtual_geometry_gpu_generated_indirect_args");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryGpuGeneratedIndirectArgs",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 3,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 0.8,
            },
        ],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
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
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(1),
                        0,
                        1,
                        2,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let narrow_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 2,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 3,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(1),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(1),
                        0,
                        2,
                        2,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let wide_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    assert_eq!(
        narrow_args,
        vec![(0, 3)],
        "expected narrow visibility-owned segment to generate a one-triangle indexed indirect draw"
    );
    assert_eq!(
        wide_args,
        vec![(0, 6)],
        "expected wide visibility-owned segment span to generate a full-quad indexed indirect draw"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_change_when_resident_slot_changes() {
    let root = unique_temp_project_root("graphics_virtual_geometry_slot_indirect_args");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySlotIndirectArgs",
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

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        green_material,
        vec![RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        }],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
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
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(1),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(1),
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 1,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let slot_one_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(7),
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let slot_seven_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    assert_ne!(
        slot_one_args, slot_seven_args,
        "expected resident-slot page-table ownership to affect GPU-generated indirect args, not just draw tint"
    );
    assert!(
        slot_seven_args[0].0 > slot_one_args[0].0 || slot_seven_args[0].1 < slot_one_args[0].1,
        "expected higher resident slot ownership to shift or trim the resident cluster raster span in GPU indirect args; slot1={slot_one_args:?}, slot7={slot_seven_args:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_change_when_page_id_changes_inside_same_resident_slot(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_page_indirect_args");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPageIndirectArgs",
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
        vec![
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 3,
                page_id: 301,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 0.9,
            },
        ],
        vec![
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: true,
                size_bytes: 4096,
            },
            RenderVirtualGeometryPage {
                page_id: 301,
                resident: true,
                size_bytes: 4096,
            },
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
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(7),
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let page_300_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 3,
                        page_id: 301,
                        lod_level: 0,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        3,
                        301,
                        Some(7),
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 301,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let page_301_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    assert_ne!(
        page_300_args, page_301_args,
        "expected different resident pages that share the same slot to still produce different GPU indirect args so deeper cluster raster consumption keeps page ownership"
    );
    assert!(
        page_301_args[0].0 > page_300_args[0].0 || page_301_args[0].1 < page_300_args[0].1,
        "expected page-owned indirect args to shift or trim the cluster raster span even when resident_slot is identical; page300={page_300_args:?}, page301={page_301_args:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_change_when_lod_level_changes_inside_same_page_slot(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_lod_indirect_args");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryLodIndirectArgs",
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
        vec![RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        }],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
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
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(7),
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let lod_zero_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 3,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(7),
                        0,
                        1,
                        1,
                        3,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let lod_three_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    assert_ne!(
        lod_zero_args, lod_three_args,
        "expected deeper cluster frontier lod_level to affect GPU-generated indirect args so real indirect submission consumes more than page/slot ownership"
    );
    assert!(
        lod_three_args[0].0 > lod_zero_args[0].0 || lod_three_args[0].1 < lod_zero_args[0].1,
        "expected deeper lod ownership to shift or trim the GPU indirect cluster raster span inside the same page/slot; lod0={lod_zero_args:?}, lod3={lod_three_args:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_changes_when_page_id_changes_inside_same_resident_slot(
) {
    let root =
        unique_temp_project_root("graphics_virtual_geometry_page_owned_cluster_raster_output");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPageOwnedClusterRasterOutput",
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
        vec![RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        }],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
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
    let page_300 = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        Some(7),
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let page_301 = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 301,
                        lod_level: 0,
                        resident_slot: Some(7),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        301,
                        Some(7),
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::Resident,
                    )],
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 301,
                        slot: 7,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let page_300_left_green = average_half_channel(&page_300.rgba, viewport_size, 1, Half::Left);
    let page_301_left_green = average_half_channel(&page_301.rgba, viewport_size, 1, Half::Left);
    let page_300_coverage = count_non_background_pixels(&page_300.rgba);
    let page_301_coverage = count_non_background_pixels(&page_301.rgba);

    assert_ne!(
        page_300.rgba, page_301.rgba,
        "expected page-owned unified indirect submission truth to change cluster raster output, not just GPU indirect args"
    );
    assert!(
        (page_300_left_green - page_301_left_green).abs() > 0.45
            || page_300_coverage.abs_diff(page_301_coverage) > 96,
        "expected page-owned submission offset to materially shift cluster raster coverage or balance; page300_left={page_300_left_green:.2}, page301_left={page_301_left_green:.2}, page300_coverage={page_300_coverage}, page301_coverage={page_301_coverage}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_changes_when_pending_request_frontier_rank_changes(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_frontier_rank_raster_output");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFrontierRankRasterOutput",
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
        vec![RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        }],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: false,
            size_bytes: 4096,
        }],
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
    let early_frontier = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: None,
                        state: VirtualGeometryPrepareClusterState::PendingUpload,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        None,
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::PendingUpload,
                    )],
                    resident_pages: Vec::new(),
                    pending_page_requests: vec![crate::types::VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 1,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let early_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();
    let late_frontier = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![2],
                    visible_clusters: vec![VirtualGeometryPrepareCluster {
                        entity: 2,
                        cluster_id: 2,
                        page_id: 300,
                        lod_level: 0,
                        resident_slot: None,
                        state: VirtualGeometryPrepareClusterState::PendingUpload,
                    }],
                    cluster_draw_segments: vec![draw_segment_with_span(
                        2,
                        2,
                        300,
                        None,
                        0,
                        1,
                        1,
                        0,
                        VirtualGeometryPrepareClusterState::PendingUpload,
                    )],
                    resident_pages: Vec::new(),
                    pending_page_requests: vec![
                        crate::types::VirtualGeometryPrepareRequest {
                            page_id: 300,
                            size_bytes: 4096,
                            generation: 2,
                            frontier_rank: 3,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        crate::types::VirtualGeometryPrepareRequest {
                            page_id: 301,
                            size_bytes: 4096,
                            generation: 3,
                            frontier_rank: 0,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        crate::types::VirtualGeometryPrepareRequest {
                            page_id: 302,
                            size_bytes: 4096,
                            generation: 4,
                            frontier_rank: 1,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        crate::types::VirtualGeometryPrepareRequest {
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
                })),
            &compiled,
            None,
        )
        .unwrap();
    let late_args = renderer.read_last_virtual_geometry_indirect_args().unwrap();

    let early_coverage = count_non_background_pixels(&early_frontier.rgba);
    let late_coverage = count_non_background_pixels(&late_frontier.rgba);

    assert_ne!(
        early_frontier.rgba, late_frontier.rgba,
        "expected pending request frontier rank to change the real rendered cluster raster output, not only the CPU-side uploader ordering"
    );
    assert_ne!(
        early_args, late_args,
        "expected pending request frontier rank to change the real GPU-generated indirect args before it changes the rendered raster output"
    );
    assert!(
        early_coverage > late_coverage + 120,
        "expected earlier pending request frontier rank to keep materially more cluster raster coverage than a later request rank; early_coverage={early_coverage}, late_coverage={late_coverage}"
    );
    assert_eq!(
        late_args.len(),
        1,
        "expected the pending frontier-rank raster regression to stay on one indirect draw"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_multi_primitive_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryMultiPrimitiveOrder",
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
    write_double_quad_gltf(
        paths.assets_root().join("models").join("double_quad_lr.gltf"),
        false,
    );
    write_double_quad_gltf(
        paths.assets_root().join("models").join("double_quad_rl.gltf"),
        true,
    );
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

    let left_then_right_model =
        resource_handle::<ModelMarker>(&asset_manager, "res://models/double_quad_lr.gltf");
    let right_then_left_model =
        resource_handle::<ModelMarker>(&asset_manager, "res://models/double_quad_rl.gltf");
    let green_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/flat_green.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let left_then_right_extract =
        build_single_entity_extract(viewport_size, left_then_right_model, green_material);
    let right_then_left_extract =
        build_single_entity_extract(viewport_size, right_then_left_model, green_material);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &left_then_right_extract,
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

    let prepare = Some(VirtualGeometryPrepareFrame {
        visible_entities: vec![2],
        visible_clusters: vec![VirtualGeometryPrepareCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            resident_slot: Some(1),
            state: VirtualGeometryPrepareClusterState::Resident,
        }],
        cluster_draw_segments: vec![draw_segment_with_span(
            2,
            2,
            300,
            Some(1),
            0,
            1,
            1,
            0,
            VirtualGeometryPrepareClusterState::Resident,
        )],
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 300,
            slot: 1,
            size_bytes: 4096,
        }],
        pending_page_requests: Vec::new(),
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    });

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let left_then_right = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(left_then_right_extract, viewport_size)
                .with_virtual_geometry_prepare(prepare.clone()),
            &compiled,
            None,
        )
        .unwrap();
    let right_then_left = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(right_then_left_extract, viewport_size)
                .with_virtual_geometry_prepare(prepare),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        left_then_right.rgba, right_then_left.rgba,
        "expected same-segment repeated primitive compaction to stay stable when only glTF primitive enumeration order changes; model import order should not keep deciding which geometry gets the compacted indirect slot"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order_with_distinct_uvs(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_multi_primitive_uv_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryMultiPrimitiveUvOrder",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_textured_color_wgsl(
        paths.assets_root().join("shaders").join("textured_color.wgsl"),
    );
    write_split_color_png(
        paths.assets_root().join("textures").join("split_rg.png"),
        [255, 32, 32, 255],
        [32, 32, 255, 255],
    );
    write_overlapping_uv_gltf(
        paths.assets_root().join("models").join("overlap_uv_lr.gltf"),
        false,
    );
    write_overlapping_uv_gltf(
        paths.assets_root().join("models").join("overlap_uv_rl.gltf"),
        true,
    );
    write_material_with_base_color_and_alpha_mode(
        paths
            .assets_root()
            .join("materials")
            .join("textured_blend.material.toml"),
        "res://shaders/textured_color.wgsl",
        "res://textures/split_rg.png",
        [1.0, 1.0, 1.0, 0.65],
        AlphaMode::Blend,
    );

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let left_then_right_model =
        resource_handle::<ModelMarker>(&asset_manager, "res://models/overlap_uv_lr.gltf");
    let right_then_left_model =
        resource_handle::<ModelMarker>(&asset_manager, "res://models/overlap_uv_rl.gltf");
    let blend_material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/textured_blend.material.toml",
    );
    let viewport_size = UVec2::new(160, 120);
    let left_then_right_extract =
        build_single_entity_extract(viewport_size, left_then_right_model, blend_material);
    let right_then_left_extract =
        build_single_entity_extract(viewport_size, right_then_left_model, blend_material);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &left_then_right_extract,
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

    let prepare = Some(VirtualGeometryPrepareFrame {
        visible_entities: vec![2],
        visible_clusters: vec![VirtualGeometryPrepareCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            resident_slot: Some(1),
            state: VirtualGeometryPrepareClusterState::Resident,
        }],
        cluster_draw_segments: vec![draw_segment_with_span(
            2,
            2,
            300,
            Some(1),
            0,
            1,
            1,
            0,
            VirtualGeometryPrepareClusterState::Resident,
        )],
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 300,
            slot: 1,
            size_bytes: 4096,
        }],
        pending_page_requests: Vec::new(),
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    });

    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let left_then_right = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(left_then_right_extract, viewport_size)
                .with_virtual_geometry_prepare(prepare.clone()),
            &compiled,
            None,
        )
        .unwrap();
    let right_then_left = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(right_then_left_extract, viewport_size)
                .with_virtual_geometry_prepare(prepare),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        left_then_right.rgba, right_then_left.rgba,
        "expected same-segment repeated primitive compaction to stay stable when only glTF primitive enumeration order changes for overlapping primitives with different texcoords; mesh import order must not keep deciding which UV-colored primitive owns the compacted indirect slot"
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

fn build_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    red_material: ResourceHandle<MaterialMarker>,
    green_material: ResourceHandle<MaterialMarker>,
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
                    node_id: 1,
                    transform: Transform {
                        translation: Vec3::new(-0.55, 0.0, 0.0),
                        scale: Vec3::new(0.55, 0.55, 1.0),
                        ..Transform::default()
                    },
                    model,
                    material: red_material,
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
                RenderMeshSnapshot {
                    node_id: 2,
                    transform: Transform {
                        translation: Vec3::new(0.55, 0.0, 0.0),
                        scale: Vec3::new(0.55, 0.55, 1.0),
                        ..Transform::default()
                    },
                    model,
                    material: green_material,
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
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity: 1,
                cluster_id: 1,
                page_id: 200,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::new(-0.55, 0.0, 0.0),
                bounds_radius: 0.8,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::new(0.55, 0.0, 0.0),
                bounds_radius: 0.8,
                screen_space_error: 1.0,
            },
        ],
        pages: vec![
            RenderVirtualGeometryPage {
                page_id: 200,
                resident: true,
                size_bytes: 2048,
            },
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: true,
                size_bytes: 4096,
            },
        ],
    });
    extract
}

fn build_single_entity_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        material,
        vec![RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: 2,
            page_id: 300,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        }],
        vec![RenderVirtualGeometryPage {
            page_id: 300,
            resident: true,
            size_bytes: 4096,
        }],
    )
}

fn build_single_entity_clustered_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    build_single_entity_extract_with_clusters(
        viewport_size,
        model,
        material,
        vec![
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity: 2,
                cluster_id: 3,
                page_id: 301,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_space_error: 0.8,
            },
        ],
        vec![
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: true,
                size_bytes: 4096,
            },
            RenderVirtualGeometryPage {
                page_id: 301,
                resident: true,
                size_bytes: 4096,
            },
        ],
    )
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
        page_budget: 1,
        clusters,
        pages,
    });
    extract
}

#[derive(Clone, Copy)]
enum Half {
    Left,
    Right,
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

fn write_textured_color_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(albedo_tex, albedo_sampler, input.uv) * model_data.tint;
}
"#,
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

fn write_split_color_png(path: PathBuf, left_rgba: [u8; 4], right_rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 1, |x, _y| {
        if x == 0 {
            Rgba(left_rgba)
        } else {
            Rgba(right_rgba)
        }
    })
    .save_with_format(path, ImageFormat::Png)
    .unwrap();
}

fn write_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 1.0 1.0 0.0
v -1.0 1.0 0.0
vt 0.0 1.0
vt 1.0 1.0
vt 1.0 0.0
vt 0.0 0.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
",
    )
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

fn write_double_quad_gltf(path: PathBuf, reverse_primitive_order: bool) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut binary = Vec::new();
    for value in [
        -1.0f32, -1.0, 0.0, //
        0.0, -1.0, 0.0, //
        0.0, 1.0, 0.0, //
        -1.0, 1.0, 0.0,
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2, 0, 2, 3] {
        binary.extend_from_slice(&index.to_le_bytes());
    }
    for value in [
        0.0f32, -1.0, 0.0, //
        1.0, -1.0, 0.0, //
        1.0, 1.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2, 0, 2, 3] {
        binary.extend_from_slice(&index.to_le_bytes());
    }

    let generated_name = if reverse_primitive_order {
        "double_quad_rl.bin"
    } else {
        "double_quad_lr.bin"
    };
    let buffer_path = path
        .ancestors()
        .nth(3)
        .expect("gltf path should live under <root>/assets/models")
        .join(".generated")
        .join(generated_name);
    if let Some(parent) = buffer_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&buffer_path, &binary).unwrap();

    let primitive_json = if reverse_primitive_order {
        r#"
        {"attributes":{"POSITION":2},"indices":3},
        {"attributes":{"POSITION":0},"indices":1}
"#
    } else {
        r#"
        {"attributes":{"POSITION":0},"indices":1},
        {"attributes":{"POSITION":2},"indices":3}
"#
    };
    let gltf = format!(
        r#"{{
  "asset": {{"version": "2.0"}},
  "buffers": [
    {{"uri": "../../.generated/{generated_name}", "byteLength": {}}}
  ],
  "bufferViews": [
    {{"buffer":0,"byteOffset":0,"byteLength":48,"target":34962}},
    {{"buffer":0,"byteOffset":48,"byteLength":24,"target":34963}},
    {{"buffer":0,"byteOffset":72,"byteLength":48,"target":34962}},
    {{"buffer":0,"byteOffset":120,"byteLength":24,"target":34963}}
  ],
  "accessors": [
    {{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[-1.0,-1.0,0.0],"max":[0.0,1.0,0.0]}},
    {{"bufferView":1,"componentType":5125,"count":6,"type":"SCALAR","min":[0],"max":[3]}},
    {{"bufferView":2,"componentType":5126,"count":4,"type":"VEC3","min":[0.0,-1.0,0.0],"max":[1.0,1.0,0.0]}},
    {{"bufferView":3,"componentType":5125,"count":6,"type":"SCALAR","min":[0],"max":[3]}}
  ],
  "meshes": [
    {{
      "primitives": [{primitive_json}
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
    fs::write(path, gltf).unwrap();
}

fn write_overlapping_uv_gltf(path: PathBuf, reverse_primitive_order: bool) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let quad_positions = [
        -1.0f32, -1.0, 0.0, //
        1.0, -1.0, 0.0, //
        1.0, 1.0, 0.0, //
        -1.0, 1.0, 0.0,
    ];
    let quad_indices = [0u32, 1, 2, 0, 2, 3];
    let left_uvs = [
        0.0f32, 1.0, //
        0.49, 1.0, //
        0.49, 0.0, //
        0.0, 0.0,
    ];
    let right_uvs = [
        0.51f32, 1.0, //
        1.0, 1.0, //
        1.0, 0.0, //
        0.51, 0.0,
    ];

    let mut binary = Vec::new();
    for value in quad_positions {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for value in left_uvs {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in quad_indices {
        binary.extend_from_slice(&index.to_le_bytes());
    }
    for value in quad_positions {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for value in right_uvs {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in quad_indices {
        binary.extend_from_slice(&index.to_le_bytes());
    }

    let generated_name = if reverse_primitive_order {
        "overlap_uv_rl.bin"
    } else {
        "overlap_uv_lr.bin"
    };
    let buffer_path = path
        .ancestors()
        .nth(3)
        .expect("gltf path should live under <root>/assets/models")
        .join(".generated")
        .join(generated_name);
    if let Some(parent) = buffer_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&buffer_path, &binary).unwrap();

    let primitive_json = if reverse_primitive_order {
        r#"
        {"attributes":{"POSITION":3,"TEXCOORD_0":4},"indices":5},
        {"attributes":{"POSITION":0,"TEXCOORD_0":1},"indices":2}
"#
    } else {
        r#"
        {"attributes":{"POSITION":0,"TEXCOORD_0":1},"indices":2},
        {"attributes":{"POSITION":3,"TEXCOORD_0":4},"indices":5}
"#
    };
    let gltf = format!(
        r#"{{
  "asset": {{"version": "2.0"}},
  "buffers": [
    {{"uri": "../../.generated/{generated_name}", "byteLength": {}}}
  ],
  "bufferViews": [
    {{"buffer":0,"byteOffset":0,"byteLength":48,"target":34962}},
    {{"buffer":0,"byteOffset":48,"byteLength":32,"target":34962}},
    {{"buffer":0,"byteOffset":80,"byteLength":24,"target":34963}},
    {{"buffer":0,"byteOffset":104,"byteLength":48,"target":34962}},
    {{"buffer":0,"byteOffset":152,"byteLength":32,"target":34962}},
    {{"buffer":0,"byteOffset":184,"byteLength":24,"target":34963}}
  ],
  "accessors": [
    {{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[-1.0,-1.0,0.0],"max":[1.0,1.0,0.0]}},
    {{"bufferView":1,"componentType":5126,"count":4,"type":"VEC2","min":[0.0,0.0],"max":[0.49,1.0]}},
    {{"bufferView":2,"componentType":5125,"count":6,"type":"SCALAR","min":[0],"max":[3]}},
    {{"bufferView":3,"componentType":5126,"count":4,"type":"VEC3","min":[-1.0,-1.0,0.0],"max":[1.0,1.0,0.0]}},
    {{"bufferView":4,"componentType":5126,"count":4,"type":"VEC2","min":[0.51,0.0],"max":[1.0,1.0]}},
    {{"bufferView":5,"componentType":5125,"count":6,"type":"SCALAR","min":[0],"max":[3]}}
  ],
  "meshes": [
    {{
      "primitives": [{primitive_json}
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
    fs::write(path, gltf).unwrap();
}

fn write_material(path: PathBuf, shader_uri: &str, texture_uri: &str) {
    write_material_with_base_color_and_alpha_mode(
        path,
        shader_uri,
        texture_uri,
        [1.0, 1.0, 1.0, 1.0],
        AlphaMode::Opaque,
    );
}

fn write_material_with_base_color_and_alpha_mode(
    path: PathBuf,
    shader_uri: &str,
    texture_uri: &str,
    base_color: [f32; 4],
    alpha_mode: AlphaMode,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("VirtualGeometryTest".to_string()),
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

fn average_channel(rgba: &[u8], channel: usize) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let total = rgba
        .chunks_exact(4)
        .map(|pixel| pixel[channel] as f32)
        .sum::<f32>();
    total / (rgba.len() as f32 / 4.0)
}

fn count_non_background_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[0] > 4 || pixel[1] > 4 || pixel[2] > 4 || pixel[3] > 4)
        .count()
}

type IndirectSegmentReadback = (
    u32,
    u32,
    u32,
    u32,
    u32,
    VirtualGeometryPrepareClusterState,
    u32,
    u32,
    u32,
);

fn indirect_segment_for_page(
    segments: &[IndirectSegmentReadback],
    page_id: u32,
) -> Option<IndirectSegmentReadback> {
    segments
        .iter()
        .copied()
        .find(|segment| segment.3 == page_id)
}

fn indirect_args_for_page(
    segments: &[IndirectSegmentReadback],
    draw_refs: &[(u32, u32)],
    indirect_args: &[(u32, u32)],
    page_id: u32,
) -> Option<(u32, u32)> {
    draw_refs
        .iter()
        .enumerate()
        .find_map(|(draw_index, draw_ref)| {
            let segment = segments.get(draw_ref.1 as usize)?;
            (segment.3 == page_id).then(|| indirect_args[draw_index])
        })
}

fn dominant_green_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| {
            pixel[1] > pixel[0].saturating_add(16) && pixel[1] > pixel[2].saturating_add(16)
        })
        .count()
}

fn average_half_channel(rgba: &[u8], viewport_size: UVec2, channel: usize, half: Half) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let x_range = match half {
        Half::Left => 0..(width / 2).max(1),
        Half::Right => (width / 2)..width,
    };

    let mut total = 0.0;
    let mut count = 0usize;
    for y in 0..height {
        for x in x_range.clone() {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }
    total / count as f32
}
