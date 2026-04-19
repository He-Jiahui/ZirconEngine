use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
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
use crate::scene::components::{default_render_layer_mask, Mobility};

use crate::{
    types::{
        EditorOrRuntimeFrame, VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

#[test]
fn virtual_geometry_args_source_keeps_prepare_owned_draw_refs_when_some_entities_do_not_emit_pending_draws(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_args_source_authority");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryArgsSourceAuthority",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model),
        (3, model),
        green_material,
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
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
                        draw_segment(2, 20, 300, 1),
                        draw_segment(3, 30, 301, 2),
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
        renderer.last_virtual_geometry_indirect_draw_count(),
        1,
        "expected only the drawable entity to submit a mesh draw even while args source authority moves up to prepare-owned visibility truth"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_draw_refs().unwrap(),
        vec![(6, 0), (6, 1)],
        "expected the shared draw-ref buffer itself to keep one prepare-owned record per visibility-owned segment, instead of collapsing back to the subset of entities that emitted pending draws"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_args().unwrap(),
        vec![(0, 6), (0, 6)],
        "expected the GPU-generated indirect args source to keep both prepare-owned records even when only one entity emitted a pending draw this frame"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_gpu_submission_fallback_ignores_non_submitted_visibility_draw_refs_when_cpu_records_are_gone(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_submission_subset_fallback");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometrySubmissionSubsetFallback",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model),
        (3, model),
        green_material,
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
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
                        draw_segment(2, 20, 300, 1),
                        draw_segment(3, 30, 301, 2),
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
        renderer
            .read_last_virtual_geometry_indirect_execution_draw_ref_indices()
            .unwrap(),
        vec![0],
        "expected renderer to publish a GPU-side submitted draw-ref index source that keeps only the actually submitted mesh draw subset before deepest readback fallback reconstructs submission ownership"
    );

    renderer.drop_last_virtual_geometry_indirect_submission_buffer_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_token_records_for_test();
    renderer.drop_last_virtual_geometry_mesh_draw_submission_records_for_test();

    assert_eq!(
        renderer
            .read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()
            .unwrap(),
        vec![(2, 300, 0, 0)],
        "expected deepest submission fallback to keep only the actual submitted mesh draws once CPU-side records are gone, instead of reviving non-submitted visibility-owned draw refs from the shared args buffers"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_fallback_full_mesh_args_follow_prepare_cluster_state_when_segments_are_absent()
{
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_full_mesh_args");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackFullMeshArgs",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model.clone()),
        (3, model),
        green_material,
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
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
                    visible_entities: vec![2, 3],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 20,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(2),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 3,
                            cluster_id: 30,
                            page_id: 301,
                            lod_level: 0,
                            resident_slot: None,
                            state: VirtualGeometryPrepareClusterState::PendingUpload,
                        },
                    ],
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 2,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: vec![crate::graphics::types::VirtualGeometryPrepareRequest {
                        page_id: 301,
                        size_bytes: 4096,
                        generation: 1,
                        frontier_rank: 0,
                        assigned_slot: Some(1),
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_segments().unwrap(),
        vec![
            (
                0,
                1,
                1,
                301,
                1,
                VirtualGeometryPrepareClusterState::PendingUpload,
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
        "expected fallback full-mesh indirect segments to inherit page/slot/state truth from prepare visibility data when cluster draw segments are absent, instead of falling back to renderer-minted page 0 / slot 0 resident defaults"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_args().unwrap(),
        vec![(3, 3), (0, 6)],
        "expected GPU-generated indirect args to follow the prepare-owned fallback segment state, so the pending-upload entity trims to one triangle while the resident entity keeps the full mesh"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_fallback_clusters_without_segments_expand_into_visibility_owned_indirect_slices(
) {
    let root = unique_temp_project_root("graphics_virtual_geometry_fallback_cluster_slices");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryFallbackClusterSlices",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model.clone()),
        (3, model),
        green_material,
        vec![
            cluster(2, 20, 300),
            cluster(2, 21, 301),
            cluster(3, 30, 302),
        ],
        vec![page(300), page(301), page(302)],
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
                            resident_slot: Some(2),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 21,
                            page_id: 301,
                            lod_level: 0,
                            resident_slot: None,
                            state: VirtualGeometryPrepareClusterState::PendingUpload,
                        },
                    ],
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 2,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: vec![crate::graphics::types::VirtualGeometryPrepareRequest {
                        page_id: 301,
                        size_bytes: 4096,
                        generation: 3,
                        frontier_rank: 0,
                        assigned_slot: Some(1),
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_segments().unwrap(),
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
                0,
            ),
            (
                0,
                1,
                2,
                300,
                2,
                VirtualGeometryPrepareClusterState::Resident,
                0,
                0,
                0,
            ),
        ],
        "expected missing cluster_draw_segments to synthesize one fallback indirect segment per visible cluster while preserving the original entity-local cluster ordinals, instead of recompacting the surviving clusters back into fresh coarse ordinals"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_draw_refs().unwrap(),
        vec![(6, 0), (6, 1)],
        "expected draw-ref submission to follow the synthesized per-cluster fallback segments once visibility-owned cluster truth takes over the fallback path"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_args().unwrap(),
        vec![(3, 3), (0, 3)],
        "expected GPU-generated indirect args to consume the synthesized per-cluster fallback ordinals in authoritative submission-slot order so each visible cluster keeps its distinct raster slice"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_mixed_explicit_and_fallback_entities_reuse_one_prepare_owned_args_source() {
    let root = unique_temp_project_root("graphics_virtual_geometry_mixed_args_source");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryMixedArgsSource",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model.clone()),
        (3, model),
        green_material,
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300), page(301)],
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
                    visible_entities: vec![2, 3],
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
                    cluster_draw_segments: vec![draw_segment(2, 20, 300, 1)],
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
        renderer.last_virtual_geometry_indirect_draw_count(),
        2,
        "expected both visible entities to submit one mesh draw when one uses explicit segments and the other reuses synthesized fallback segments"
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
        "expected mixed explicit and synthesized fallback entities to share the same prepare-owned segment authority, without a duplicate fallback-only segment record minted later in mesh-build"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_draw_refs().unwrap(),
        vec![(6, 0), (6, 1)],
        "expected the shared draw-ref buffer to keep only the two prepare-owned records, instead of appending a duplicate fallback-only draw-ref for the synthesized entity"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_args().unwrap(),
        vec![(0, 6), (0, 6)],
        "expected the GPU-generated args source to stay identical to the prepare-owned unified indirect truth when explicit and fallback entities coexist"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_missing_explicit_segments_do_not_resurrect_cpu_full_mesh_fallback_draws() {
    let root = unique_temp_project_root("graphics_virtual_geometry_missing_segment_authority");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryMissingSegmentAuthority",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model.clone()),
        (3, model),
        green_material,
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: false,
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
                    visible_entities: vec![2, 3],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 20,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: None,
                            state: VirtualGeometryPrepareClusterState::Missing,
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
                        VirtualGeometryPrepareDrawSegment {
                            entity: 2,
                            cluster_id: 20,
                            page_id: 300,
                            resident_slot: None,
                            cluster_ordinal: 0,
                            cluster_span_count: 1,
                            cluster_count: 1,
                            lineage_depth: 0,
                            lod_level: 0,
                            state: VirtualGeometryPrepareClusterState::Missing,
                        },
                        VirtualGeometryPrepareDrawSegment {
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

    assert_eq!(
        renderer.last_virtual_geometry_indirect_draw_count(),
        1,
        "expected a Missing explicit segment to suppress that entity all the way through actual submission instead of resurrecting a CPU-minted full-mesh fallback draw at the end of mesh-build"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_segments().unwrap(),
        vec![(
            0,
            1,
            1,
            301,
            2,
            VirtualGeometryPrepareClusterState::Resident,
            0,
            0,
            0,
        )],
        "expected the real GPU-submitted segment buffer to keep only the authoritative resident segment once a sibling entity's explicit segment has collapsed to Missing"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_draw_refs().unwrap(),
        vec![(6, 0)],
        "expected draw-ref generation to stay anchored to the prepare-owned authoritative source instead of appending a fallback full-mesh record for the Missing segment entity"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_args().unwrap(),
        vec![(0, 6)],
        "expected GPU-generated indirect args to remain empty for the Missing explicit segment entity rather than reviving a page-0 slot-0 fallback slice"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_missing_fallback_clusters_do_not_emit_zero_count_indirect_records() {
    let root = unique_temp_project_root("graphics_virtual_geometry_missing_fallback_clusters");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryMissingFallbackClusters",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model.clone()),
        (3, model),
        green_material,
        vec![cluster(2, 20, 300), cluster(2, 21, 301)],
        vec![
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: false,
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
                            cluster_id: 20,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: None,
                            state: VirtualGeometryPrepareClusterState::Missing,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 21,
                            page_id: 301,
                            lod_level: 0,
                            resident_slot: Some(2),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: Vec::new(),
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

    assert_eq!(
        renderer.last_virtual_geometry_indirect_draw_count(),
        1,
        "expected synthesized fallback cluster draws to treat Missing clusters as authoritative no-draw truth instead of emitting a zero-count ghost indirect draw record"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_segments().unwrap(),
        vec![(
            1,
            1,
            2,
            301,
            2,
            VirtualGeometryPrepareClusterState::Resident,
            0,
            0,
            0,
        )],
        "expected the real GPU-submitted segment buffer to drop Missing fallback clusters while preserving the surviving cluster's original entity-local ordinal and total-count truth"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_draw_refs().unwrap(),
        vec![(6, 0)],
        "expected draw-ref generation to skip Missing fallback clusters once no-draw truth is preserved through the synthesized indirect path"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_partial_missing_fallback_clusters_keep_original_cluster_ordinal_in_gpu_args() {
    let root = unique_temp_project_root("graphics_virtual_geometry_partial_missing_fallback");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryPartialMissingFallback",
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
    let extract = build_dual_entity_extract_with_clusters(
        viewport_size,
        (2, model.clone()),
        (3, model),
        green_material,
        vec![cluster(2, 20, 300), cluster(2, 21, 301)],
        vec![
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: false,
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
                            cluster_id: 20,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: None,
                            state: VirtualGeometryPrepareClusterState::Missing,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: 2,
                            cluster_id: 21,
                            page_id: 301,
                            lod_level: 0,
                            resident_slot: Some(2),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: Vec::new(),
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

    assert_eq!(
        renderer.last_virtual_geometry_indirect_draw_count(),
        1,
        "expected only the surviving resident fallback cluster to submit after a sibling Missing cluster collapses to no-draw"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_segments().unwrap(),
        vec![(
            1,
            1,
            2,
            301,
            2,
            VirtualGeometryPrepareClusterState::Resident,
            0,
            0,
            0,
        )],
        "expected synthesized fallback segments to preserve the original entity-local cluster ordinal and total cluster count once a sibling Missing cluster is filtered out, instead of recompacting the surviving cluster back to a full-mesh slice"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_draw_refs().unwrap(),
        vec![(6, 0)],
        "expected draw-ref generation to keep one authoritative record for the surviving fallback cluster after Missing siblings collapse out"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_indirect_args().unwrap(),
        vec![(3, 3)],
        "expected GPU-generated indirect args to keep the surviving fallback cluster in the second half of the entity mesh once Missing siblings collapse out, instead of expanding it back to a full-mesh draw"
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

fn write_material(path: PathBuf, shader_uri: &str, texture_uri: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("VirtualGeometryArgsSourceAuthority".to_string()),
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
