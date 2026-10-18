use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::assets::{AlphaMode, MaterialAsset};
use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::{
    cook_virtual_geometry_from_mesh, AssetManager, AssetReference, AssetUri, MeshVertex,
    ModelAsset, ModelPrimitiveAsset, ProjectManifest, VirtualGeometryCookConfig,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, DisplayMode,
    FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState,
    RenderOverlayExtract, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderViewportDescriptor, RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryInstance, RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::graphics::{
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions,
};
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

use crate::{
    test_support::render_feature_fixtures::{
        pluginized_wgpu_render_framework_with_asset_manager,
        virtual_geometry_render_feature_descriptor,
    },
    virtual_geometry::{
        VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    },
};

const RTX_3060_ADAPTER_NAME: &str = "NVIDIA GeForce RTX 3060";
const VIRTUAL_GEOMETRY_EXECUTOR_IDS: [&str; 5] = [
    "virtual-geometry.prepare",
    "virtual-geometry.node-cluster-cull",
    "virtual-geometry.page-feedback",
    "virtual-geometry.visbuffer",
    "virtual-geometry.debug-overlay",
];

#[test]
fn render_framework_executes_virtual_geometry_gpu_chain_with_resident_payload() {
    let (root, asset_manager, model) = cooked_triangle_project();
    let material = resource_handle::<MaterialMarker>(&asset_manager, "builtin://material/default");
    let viewport_size = UVec2::new(160, 120);
    let mut extract = build_single_entity_extract(viewport_size, model, material);
    extract.geometry.virtual_geometry = None;
    extract.geometry.virtual_geometry_debug = Some(RenderVirtualGeometryDebugState {
        visualize_visbuffer: true,
        ..Default::default()
    });
    let stable_extract = extract.clone();

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, virtual_geometry_only_quality_profile())
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();

    let stats = server.query_stats().unwrap();
    let adapter_name = &stats
        .device_diagnostics
        .as_ref()
        .expect("virtual geometry GPU contract requires device diagnostics")
        .adapter_name;
    assert!(
        adapter_name.contains(RTX_3060_ADAPTER_NAME),
        "virtual geometry GPU contract requires `{RTX_3060_ADAPTER_NAME}`, got `{adapter_name}`"
    );
    let executed_virtual_geometry = stats
        .last_graph_executed_executor_ids
        .iter()
        .filter(|executor_id| executor_id.starts_with("virtual-geometry."))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(executed_virtual_geometry, VIRTUAL_GEOMETRY_EXECUTOR_IDS);
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 5);
    assert!(stats.last_graph_compute_planned_workload_count > 0);
    assert_eq!(
        stats.last_graph_compute_planned_workload_count,
        stats.last_graph_compute_matched_workload_count
    );
    assert_eq!(stats.last_graph_compute_missing_dispatch_count, 0);
    assert_eq!(stats.last_graph_compute_workload_mismatch_count, 0);
    assert_eq!(stats.last_graph_compute_unexpected_dispatch_count, 0);
    let cull_profile = stats
        .last_graph_execution_profile_report
        .pass_profiles
        .iter()
        .find(|profile| profile.executor_id == "virtual-geometry.node-cluster-cull")
        .expect("node/cluster cull pass profile should be recorded");
    assert_eq!(cull_profile.dispatch_count, 1);

    server
        .submit_frame_extract(viewport, stable_extract)
        .unwrap();
    let stable_stats = server.query_stats().unwrap();
    let stable_virtual_geometry = stable_stats
        .last_graph_executed_executor_ids
        .iter()
        .filter(|executor_id| executor_id.starts_with("virtual-geometry."))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(stable_virtual_geometry, VIRTUAL_GEOMETRY_EXECUTOR_IDS);
    assert_eq!(
        stable_stats.last_graph_compute_planned_workload_count,
        stable_stats.last_graph_compute_matched_workload_count
    );
    assert_eq!(stable_stats.last_graph_compute_missing_dispatch_count, 0);
    assert_eq!(stable_stats.last_graph_compute_workload_mismatch_count, 0);
    assert_eq!(stable_stats.last_graph_compute_unexpected_dispatch_count, 0);

    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .unwrap()
        .expect("automatic virtual geometry extract should produce a debug snapshot");
    assert_eq!(snapshot.resident_page_payloads.len(), 1);
    assert_eq!(snapshot.resident_page_payloads[0].vertices.len(), 3);
    assert_eq!(snapshot.resident_page_payloads[0].cluster_ranges.len(), 1);
    assert_eq!(snapshot.selected_clusters.len(), 1);
    assert_eq!(
        snapshot.resident_page_payloads[0].cluster_ranges[0].cluster_id,
        snapshot.selected_clusters[0].cluster_id
    );

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("virtual geometry graph should produce a captured frame");
    assert!(
        frame.rgba.chunks_exact(4).any(|pixel| {
            pixel[1] > pixel[0].saturating_add(20) && pixel[1] > pixel[2].saturating_add(20)
        }),
        "visbuffer overlay should tint at least one submitted framebuffer pixel green"
    );

    drop(server);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_framework_uploads_multiple_cooked_virtual_geometry_pages() {
    let (root, asset_manager, model) = cooked_two_page_project();
    let material = resource_handle::<MaterialMarker>(&asset_manager, "builtin://material/default");
    let viewport_size = UVec2::new(160, 120);
    let mut extract = build_single_entity_extract(viewport_size, model, material);
    // Exercise automatic extraction from a cooked model so page payloads are supplied by the
    // registered runtime provider and pass through the real GPU upload path.
    extract.geometry.virtual_geometry = None;
    extract.geometry.virtual_geometry_debug = Some(RenderVirtualGeometryDebugState {
        forced_mip: Some(10),
        ..Default::default()
    });

    let framework = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, virtual_geometry_only_quality_profile())
        .unwrap();
    let mut completed_page_count = 0;
    let mut replaced_page_count = 0;
    for _ in 0..4 {
        framework
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
        let stats = framework.query_stats().unwrap();
        assert_eq!(stats.last_virtual_geometry_page_budget, 1);
        assert!(stats.last_virtual_geometry_requested_page_count <= 1);
        completed_page_count += stats.last_virtual_geometry_completed_page_count;
        replaced_page_count += stats.last_virtual_geometry_replaced_page_count;
    }
    assert!(
        completed_page_count >= 2,
        "multi-page upload should complete both nonresident leaf pages across budgeted frames"
    );
    assert!(
        replaced_page_count >= 1,
        "one-slot residency should report page replacement while the streaming frontier advances"
    );
    let snapshot = framework
        .query_virtual_geometry_debug_snapshot()
        .unwrap()
        .expect("automatic extraction should produce a debug snapshot");
    assert!(!snapshot.resident_page_payloads.is_empty());
    assert_eq!(snapshot.resident_pages.len(), 1);
    assert_eq!(snapshot.selected_clusters.len(), 1);
    framework
        .capture_frame(viewport)
        .unwrap()
        .expect("multi-page virtual geometry should produce a captured frame");
    framework.destroy_viewport(viewport).unwrap();

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_framework_stats_follow_public_virtual_geometry_execution_segments() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = resource_handle::<ModelMarker>(&asset_manager, "builtin://cube");
    let material = resource_handle::<MaterialMarker>(&asset_manager, "builtin://material/default");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_single_entity_extract(viewport_size, model, material);

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, virtual_geometry_only_quality_profile())
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();

    let stats = server.query_stats().unwrap();
    assert_eq!(
        stats.last_virtual_geometry_indirect_draw_count, 1,
        "expected public RenderFramework stats to count the visibility-owned VG execution segment produced by the authored cluster"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_segment_count, 1,
        "expected render-framework stats to expose the single execution segment selected from the authored VG extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_page_count, 1,
        "expected render-framework stats to expose the execution subset page count rather than only the prepare-owned page universe"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_resident_segment_count, 1,
        "expected render-framework stats to classify the authored cluster execution segment as resident"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_pending_segment_count, 0,
        "expected render-framework stats to keep pending-upload execution counts at zero for the fully resident authored extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_missing_segment_count, 0,
        "expected render-framework stats to keep missing execution counts at zero for the fully resident authored extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_repeated_draw_count, 0,
        "expected the authored single-cluster VG extract to report no repeated execution draw expansion"
    );
    assert_eq!(
        stats.last_virtual_geometry_selected_cluster_count, 1,
        "expected render-framework stats to expose the executed selected-cluster count"
    );
}

#[test]
fn render_framework_stats_expose_repeated_virtual_geometry_execution_draws() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = resource_handle::<ModelMarker>(&asset_manager, "builtin://cube");
    let material = resource_handle::<MaterialMarker>(&asset_manager, "builtin://material/default");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_same_page_dual_entity_extract(viewport_size, model, material);

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, virtual_geometry_only_quality_profile())
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();

    let stats = server.query_stats().unwrap();
    assert_eq!(
        stats.last_virtual_geometry_indirect_draw_count, 2,
        "expected the public RenderFramework stats to expose both same-page VG execution draws"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_segment_count, 2,
        "expected same-page execution segments to stay visible through the public RenderFramework stats seam"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_page_count, 1,
        "expected execution page accounting to compact repeated same-page segments through the neutral stats DTO"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_resident_segment_count, 2,
        "expected both same-page execution segments to classify as resident"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_pending_segment_count, 0,
        "expected no pending-upload execution segments for the fully resident authored extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_missing_segment_count, 0,
        "expected no missing execution segments for the fully resident authored extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_repeated_draw_count, 1,
        "expected public stats to surface the repeated same-page draw count without renderer-private readback helpers"
    );
    assert_eq!(
        stats.last_virtual_geometry_selected_cluster_count, 2,
        "expected selected-cluster stats to follow execution span count rather than the compacted page count"
    );
}

#[test]
fn render_framework_stats_expose_node_and_cluster_cull_worklist_counts() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = resource_handle::<ModelMarker>(&asset_manager, "builtin://cube");
    let material = resource_handle::<MaterialMarker>(&asset_manager, "builtin://material/default");
    let viewport_size = UVec2::new(160, 120);
    let extract = build_hierarchical_instance_extract(viewport_size, model, material);

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, virtual_geometry_only_quality_profile())
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();

    let stats = server.query_stats().unwrap();
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_source,
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        "expected public RenderFramework stats to expose the neutral NodeAndClusterCull startup source"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_record_count, 1,
        "expected one startup record for the authored VG extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_dispatch_group_count,
        [1, 1, 1],
        "expected one dispatch group for the single authored instance"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_instance_seed_count, 1,
        "expected the instance seed count to follow the authored neutral VG instance list"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_instance_work_item_count, 1,
        "expected public stats to expose one root instance work item without direct renderer readback"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count, 2,
        "expected the authored instance cluster range to expand into two cluster work items"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count, 2,
        "expected the authored hierarchy child-id table to cross the public stats seam"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_child_work_item_count, 2,
        "expected two authored child work items from the root hierarchy node"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_traversal_record_count, 9,
        "expected traversal accounting to reflect root visits, authored children, and leaf store probes"
    );
}

pub(super) fn build_single_entity_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ortho_size: 1.2,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![mesh_snapshot(
                2,
                Transform {
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                model,
                material,
            )],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: Default::default(),
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

pub(super) fn build_same_page_dual_entity_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ortho_size: 1.2,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                mesh_snapshot(
                    2,
                    Transform {
                        translation: Vec3::new(-0.35, 0.0, 0.0),
                        scale: Vec3::new(0.6, 0.6, 1.0),
                        ..Transform::default()
                    },
                    model.clone(),
                    material.clone(),
                ),
                mesh_snapshot(
                    3,
                    Transform {
                        translation: Vec3::new(0.35, 0.0, 0.0),
                        scale: Vec3::new(0.6, 0.6, 1.0),
                        ..Transform::default()
                    },
                    model,
                    material,
                ),
            ],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: Default::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(2), snapshot);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![cluster(2, 20, 300), cluster(3, 30, 300)],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(300, true)],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    extract
}

pub(super) fn build_hierarchical_instance_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ortho_size: 1.2,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![mesh_snapshot(
                2,
                Transform {
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                model,
                material,
            )],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: Default::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(3), snapshot);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            hierarchy_cluster(2, 100, 300, Some(1), None),
            hierarchy_cluster(2, 200, 301, Some(2), Some(100)),
        ],
        hierarchy_nodes: vec![
            hierarchy_node(0, 1, 0, 2, 0, 2),
            hierarchy_node(0, 2, 0, 0, 1, 1),
            hierarchy_node(0, 3, 0, 0, 1, 1),
        ],
        hierarchy_child_ids: vec![2, 3],
        pages: vec![page(300, true), page(301, true)],
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity: 2,
            stable_instance_key: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("hierarchical-test-mesh".to_string()),
            source_hint: Some("public-render-framework-stats".to_string()),
        }],
        debug: Default::default(),
    });
    extract
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

fn page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

fn hierarchy_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    hierarchy_node_id: Option<u32>,
    parent_cluster_id: Option<u32>,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id,
        page_id,
        lod_level: 0,
        parent_cluster_id,
        bounds_center: Vec3::ZERO,
        bounds_radius: 1.0,
        screen_space_error: 1.0,
    }
}

fn hierarchy_node(
    instance_index: u32,
    node_id: u32,
    child_base: u32,
    child_count: u32,
    cluster_start: u32,
    cluster_count: u32,
) -> RenderVirtualGeometryHierarchyNode {
    RenderVirtualGeometryHierarchyNode {
        instance_index,
        node_id,
        child_base,
        child_count,
        cluster_start,
        cluster_count,
    }
}

fn mesh_snapshot(
    node_id: u64,
    transform: Transform,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            is_static: false,
            ..RendererCommon::default()
        },
    }
}

pub(super) fn virtual_geometry_only_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("vg-execution-stats")
        .with_virtual_geometry(true)
        .with_hybrid_global_illumination(false)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_async_compute(false)
}

fn cooked_triangle_project() -> (
    PathBuf,
    Arc<ProjectAssetManager>,
    ResourceHandle<ModelMarker>,
) {
    let root = unique_temp_graphics_root("vg_render_graph");
    let model_dir = root.join("assets").join("models");
    fs::create_dir_all(&model_dir).unwrap();
    ProjectManifest::new(
        "VirtualGeometryRenderGraph",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();

    let uri = AssetUri::parse("res://models/vg_triangle.model.toml").unwrap();
    let vertices = vec![
        MeshVertex::new(Vec3::new(-0.8, -0.8, 0.0), Vec3::Z, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(0.8, -0.8, 0.0), Vec3::Z, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(0.0, 0.8, 0.0), Vec3::Z, Vec2::new(0.5, 0.0)),
    ];
    let indices = vec![0, 1, 2];
    let virtual_geometry = cook_virtual_geometry_from_mesh(
        &vertices,
        &indices,
        VirtualGeometryCookConfig {
            cluster_triangle_count: 1,
            page_cluster_count: 1,
            mesh_name: Some("render-graph-triangle".to_string()),
            source_hint: Some(uri.to_string()),
        },
    )
    .expect("single triangle should produce virtual geometry");
    let model_asset = ModelAsset {
        uri: uri.clone(),
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices,
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: Some(virtual_geometry),
        }],
    };
    fs::write(
        model_dir.join("vg_triangle.model.toml"),
        toml::to_string_pretty(&model_asset).unwrap(),
    )
    .unwrap();

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let model_uri = uri.to_string();
    let model = resource_handle::<ModelMarker>(&asset_manager, &model_uri);
    (root, asset_manager, model)
}

fn cooked_two_page_project() -> (
    PathBuf,
    Arc<ProjectAssetManager>,
    ResourceHandle<ModelMarker>,
) {
    let root = unique_temp_graphics_root("vg_render_graph_two_pages");
    let model_dir = root.join("assets").join("models");
    fs::create_dir_all(&model_dir).unwrap();
    ProjectManifest::new(
        "VirtualGeometryTwoPageRenderGraph",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();

    let uri = AssetUri::parse("res://models/vg_two_pages.model.toml").unwrap();
    let vertices = vec![
        MeshVertex::new(Vec3::new(-0.95, -0.8, 0.0), Vec3::Z, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(-0.05, -0.8, 0.0), Vec3::Z, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(-0.5, 0.8, 0.0), Vec3::Z, Vec2::new(0.5, 0.0)),
        MeshVertex::new(Vec3::new(0.05, -0.8, 0.0), Vec3::Z, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(0.95, -0.8, 0.0), Vec3::Z, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, 0.8, 0.0), Vec3::Z, Vec2::new(0.5, 0.0)),
    ];
    let indices = vec![0, 1, 2, 3, 4, 5];
    let virtual_geometry = cook_virtual_geometry_from_mesh(
        &vertices,
        &indices,
        VirtualGeometryCookConfig {
            cluster_triangle_count: 1,
            page_cluster_count: 1,
            mesh_name: Some("render-graph-two-page-mesh".to_string()),
            source_hint: Some(uri.to_string()),
        },
    )
    .expect("two triangles should cook into two virtual geometry pages");
    let model_asset = ModelAsset {
        uri: uri.clone(),
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices,
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: Some(virtual_geometry),
        }],
    };
    fs::write(
        model_dir.join("vg_two_pages.model.toml"),
        toml::to_string_pretty(&model_asset).unwrap(),
    )
    .unwrap();

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let model = resource_handle::<ModelMarker>(&asset_manager, &uri.to_string());
    (root, asset_manager, model)
}

fn unique_temp_graphics_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_graphics_{label}_{unique}"))
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource id for {uri}")),
    )
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order(
) {
    let root = unique_temp_graphics_root("graphics_virtual_geometry_multi_primitive_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryMultiPrimitiveOrder",
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
    write_double_quad_gltf(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("double_quad_lr.gltf"),
        false,
    );
    write_double_quad_gltf(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("double_quad_rl.gltf"),
        true,
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

    let left_then_right_model =
        resource_handle::<ModelMarker>(&asset_manager, "res://models/double_quad_lr.gltf");
    let right_then_left_model =
        resource_handle::<ModelMarker>(&asset_manager, "res://models/double_quad_rl.gltf");
    let green_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_green.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let left_then_right_extract =
        build_single_entity_extract(viewport_size, left_then_right_model, green_material);
    let right_then_left_extract =
        build_single_entity_extract(viewport_size, right_then_left_model, green_material);
    let compiled = compile_virtual_geometry_pipeline(&left_then_right_extract);

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

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let ((_, left_rgba, _), (_, right_rgba, _)) = renderer
        .render_compiled_virtual_geometry_captured_test_frame_pair(
            viewport_size,
            (left_then_right_extract, prepare.clone()),
            (right_then_left_extract, prepare),
            &compiled,
        )
        .unwrap();
    let left_then_right = ActualCapturedVirtualGeometryFrame { rgba: left_rgba };
    let right_then_left = ActualCapturedVirtualGeometryFrame { rgba: right_rgba };

    assert_eq!(
        left_then_right.rgba, right_then_left.rgba,
        "expected same-segment repeated primitive compaction to stay stable when only glTF primitive enumeration order changes; model import order should not keep deciding which geometry gets the compacted indirect slot"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order_with_distinct_uvs(
) {
    let root = unique_temp_graphics_root("graphics_virtual_geometry_multi_primitive_uv_order");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "GraphicsVirtualGeometryMultiPrimitiveUvOrder",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_textured_color_wgsl(
        paths
            .asset_root(&RelPath::project_assets())
            .join("shaders")
            .join("textured_color.wgsl"),
    );
    write_split_color_png(
        paths
            .asset_root(&RelPath::project_assets())
            .join("textures")
            .join("split_rg.png"),
        [255, 32, 32, 255],
        [32, 32, 255, 255],
    );
    write_overlapping_uv_gltf(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("overlap_uv_lr.gltf"),
        false,
    );
    write_overlapping_uv_gltf(
        paths
            .asset_root(&RelPath::project_assets())
            .join("models")
            .join("overlap_uv_rl.gltf"),
        true,
    );
    write_material_with_base_color_and_alpha_mode(
        paths
            .asset_root(&RelPath::project_assets())
            .join("materials")
            .join("textured_blend.zmaterial"),
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
        "res://materials/textured_blend.zmaterial",
    );
    let viewport_size = UVec2::new(160, 120);
    let left_then_right_extract =
        build_single_entity_extract(viewport_size, left_then_right_model, blend_material);
    let right_then_left_extract =
        build_single_entity_extract(viewport_size, right_then_left_model, blend_material);
    let compiled = compile_virtual_geometry_pipeline(&left_then_right_extract);

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

    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let ((_, left_rgba, _), (_, right_rgba, _)) = renderer
        .render_compiled_virtual_geometry_captured_test_frame_pair(
            viewport_size,
            (left_then_right_extract, prepare.clone()),
            (right_then_left_extract, prepare),
            &compiled,
        )
        .unwrap();
    let left_then_right = ActualCapturedVirtualGeometryFrame { rgba: left_rgba };
    let right_then_left = ActualCapturedVirtualGeometryFrame { rgba: right_rgba };

    assert_eq!(
        left_then_right.rgba, right_then_left.rgba,
        "expected same-segment repeated primitive compaction to stay stable when only glTF primitive enumeration order changes for overlapping primitives with different texcoords; mesh import order must not keep deciding which UV-colored primitive owns the compacted indirect slot"
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

fn virtual_geometry_scene_renderer(
    asset_manager: Arc<ProjectAssetManager>,
) -> crate::test_support::render_feature_fixtures::PluginizedWgpuRenderFrameworkFixture {
    crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_uploader_collector(
        asset_manager, Arc::new(crate::virtual_geometry::VirtualGeometryUploaderTestCollector::default()),
    )
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
) -> crate::virtual_geometry::types::VirtualGeometryPrepareDrawSegment {
    crate::virtual_geometry::types::VirtualGeometryPrepareDrawSegment {
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

struct ActualCapturedVirtualGeometryFrame {
    rgba: Vec<u8>,
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
