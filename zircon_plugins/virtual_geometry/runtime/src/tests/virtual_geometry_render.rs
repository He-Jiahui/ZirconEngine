use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::{
    cook_virtual_geometry_from_mesh, AssetManager, AssetUri, MeshVertex, ModelAsset,
    ModelPrimitiveAsset, ProjectManifest, VirtualGeometryCookConfig,
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
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

use crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_asset_manager;

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
        stats.last_virtual_geometry_execution_segment_count,
        1,
        "expected render-framework stats to expose the single execution segment selected from the authored VG extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_page_count,
        1,
        "expected render-framework stats to expose the execution subset page count rather than only the prepare-owned page universe"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_resident_segment_count,
        1,
        "expected render-framework stats to classify the authored cluster execution segment as resident"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_pending_segment_count,
        0,
        "expected render-framework stats to keep pending-upload execution counts at zero for the fully resident authored extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_missing_segment_count,
        0,
        "expected render-framework stats to keep missing execution counts at zero for the fully resident authored extract"
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_repeated_draw_count,
        0,
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

fn build_same_page_dual_entity_extract(
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

fn build_hierarchical_instance_extract(
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

fn virtual_geometry_only_quality_profile() -> RenderQualityProfile {
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
        model_asset.to_toml_string().unwrap(),
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
