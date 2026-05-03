use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderMeshSnapshot, RenderOverlayExtract, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportDescriptor,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryInstance, RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

use crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_asset_manager;

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
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                model,
                material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            }],
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
                    node_id: 2,
                    transform: Transform {
                        translation: Vec3::new(-0.35, 0.0, 0.0),
                        scale: Vec3::new(0.6, 0.6, 1.0),
                        ..Transform::default()
                    },
                    model: model.clone(),
                    material: material.clone(),
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                },
                RenderMeshSnapshot {
                    node_id: 3,
                    transform: Transform {
                        translation: Vec3::new(0.35, 0.0, 0.0),
                        scale: Vec3::new(0.6, 0.6, 1.0),
                        ..Transform::default()
                    },
                    model,
                    material,
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
                    translation: Vec3::ZERO,
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..Transform::default()
                },
                model,
                material,
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                render_layer_mask: default_render_layer_mask(),
            }],
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

fn virtual_geometry_only_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("vg-execution-stats")
        .with_virtual_geometry(true)
        .with_hybrid_global_illumination(false)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_history_resolve(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_async_compute(false)
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource id for {uri}")),
    )
}
