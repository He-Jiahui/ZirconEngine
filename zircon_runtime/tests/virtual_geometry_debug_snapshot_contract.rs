use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::pipeline::manager::AssetManager;
use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::pipeline::types::MeshVertex;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{
    AssetUri, ModelAsset, ModelPrimitiveAsset, SceneAsset, VirtualGeometryAsset,
    VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryRootClusterRangeAsset,
};
use zircon_runtime::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderMeshSnapshot, RenderQualityProfile,
    RenderSceneSnapshot, RenderViewportDescriptor, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryBvhVisualizationNode, RenderVirtualGeometryCluster,
    RenderVirtualGeometryCpuReferenceDepthClusterMapEntry,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryCpuReferenceLeafCluster,
    RenderVirtualGeometryCpuReferenceMipClusterMapEntry,
    RenderVirtualGeometryCpuReferenceNodeVisit,
    RenderVirtualGeometryCpuReferencePageClusterMapEntry,
    RenderVirtualGeometryCpuReferenceSelectedCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageRequestInspection, RenderVirtualGeometryResidentPageInspection,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometryVisBufferMark,
    RenderWorldSnapshotHandle,
};
use zircon_runtime::core::math::{view_matrix, Mat4, Transform, UVec2, Vec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};
use zircon_runtime::scene::world::World;

#[test]
fn render_framework_exposes_virtual_geometry_debug_snapshot_for_effective_visible_clusters() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).expect("framework should initialize");
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-debug-snapshot")
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
                .with_async_compute(false),
        )
        .expect("quality profile should be accepted");

    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let instance = RenderVirtualGeometryInstance {
        entity: mesh,
        source_model: None,
        transform: Transform::default(),
        cluster_offset: 1,
        cluster_count: 2,
        page_offset: 1,
        page_count: 2,
        mesh_name: Some("DebugSnapshotMesh".to_string()),
        source_hint: Some("integration-test".to_string()),
    };
    let debug = RenderVirtualGeometryDebugState {
        forced_mip: Some(10),
        visualize_bvh: true,
        visualize_visbuffer: true,
        print_leaf_clusters: true,
        ..RenderVirtualGeometryDebugState::default()
    };
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    let expected_camera_transform = extract.view.camera.transform;
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 8,
        page_budget: 2,
        clusters: vec![
            virtual_geometry_cluster(mesh, 10, 100, 0, Vec3::new(0.0, 0.0, 0.0), 9.0),
            virtual_geometry_cluster(mesh, 20, 200, 10, Vec3::new(0.1, 0.0, 0.0), 8.0),
            virtual_geometry_cluster(mesh, 30, 300, 10, Vec3::new(0.2, 0.0, 0.0), 7.0),
        ],
        pages: vec![
            virtual_geometry_page(100, false),
            virtual_geometry_page(200, false),
            virtual_geometry_page(300, true),
        ],
        instances: vec![instance.clone()],
        debug,
    });
    let expected_view_proj = Mat4::perspective_rh(
        extract.view.camera.fov_y_radians,
        viewport_size.x as f32 / viewport_size.y as f32,
        extract.view.camera.z_near,
        extract.view.camera.z_far,
    )
    .mul_mat4(&view_matrix(expected_camera_transform))
    .to_cols_array_2d();

    server
        .submit_frame_extract(viewport, extract)
        .expect("virtual geometry submission should succeed");

    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot query should succeed")
        .expect("virtual geometry snapshot should be present");
    let stats = server.query_stats().expect("stats query should succeed");
    assert_eq!(snapshot.instances, vec![instance]);
    assert_eq!(snapshot.debug, debug);
    assert_eq!(snapshot.visible_cluster_ids, vec![20, 30]);
    assert_eq!(snapshot.requested_pages, vec![200]);
    assert_eq!(snapshot.resident_pages, vec![300]);
    assert!(snapshot.dirty_requested_pages.contains(&200));
    assert_eq!(
        snapshot.resident_page_inspections,
        vec![RenderVirtualGeometryResidentPageInspection {
            page_id: 300,
            slot: 0,
            size_bytes: 4096,
        }]
    );
    assert_eq!(
        snapshot.pending_page_request_inspections,
        vec![RenderVirtualGeometryPageRequestInspection {
            page_id: 200,
            size_bytes: 4096,
            generation: 1,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: None,
        }]
    );
    assert_eq!(snapshot.available_page_slots, vec![1]);
    assert_eq!(snapshot.evictable_page_inspections, Vec::new());
    assert_eq!(
        snapshot.execution_segment_count as usize,
        stats.last_virtual_geometry_execution_segment_count
    );
    assert_eq!(
        snapshot.execution_page_count as usize,
        stats.last_virtual_geometry_execution_page_count
    );
    assert_eq!(
        snapshot.execution_resident_segment_count as usize,
        stats.last_virtual_geometry_execution_resident_segment_count
    );
    assert_eq!(
        snapshot.execution_pending_segment_count as usize,
        stats.last_virtual_geometry_execution_pending_segment_count
    );
    assert_eq!(
        snapshot.execution_missing_segment_count as usize,
        stats.last_virtual_geometry_execution_missing_segment_count
    );
    assert_eq!(
        snapshot.execution_repeated_draw_count as usize,
        stats.last_virtual_geometry_execution_repeated_draw_count
    );
    assert_eq!(
        snapshot.execution_indirect_offsets.len(),
        stats.last_virtual_geometry_indirect_draw_count
    );
    assert_eq!(
        snapshot.execution_segments.len() as usize,
        stats.last_virtual_geometry_indirect_draw_count
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_source,
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        "expected the public VG debug snapshot to expose the same first-pass NodeAndClusterCull startup provenance as the renderer-owned pass seam"
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_record_count,
        1,
        "expected the public VG debug snapshot to expose one NodeAndClusterCull startup record for the effective VG extract"
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_source,
        stats.last_virtual_geometry_node_and_cluster_cull_source,
        "expected the public VG debug snapshot and public RenderStats surface to agree on NodeAndClusterCull startup provenance"
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_record_count as usize,
        stats.last_virtual_geometry_node_and_cluster_cull_record_count,
        "expected the public VG debug snapshot and public RenderStats surface to agree on NodeAndClusterCull startup record count"
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_global_state,
        Some(RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
            cull_input: snapshot.cull_input,
            viewport_size: [viewport_size.x, viewport_size.y],
            camera_translation: expected_camera_transform.translation.to_array(),
            view_proj: expected_view_proj,
        }),
        "expected the public VG debug snapshot to expose the typed NodeAndClusterCull global-state record so host tooling can inspect viewport, camera origin, and view-projection inputs without renderer-private readback helpers"
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_instance_seeds,
        vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
            instance_index: 0,
            entity: mesh,
            cluster_offset: 1,
            cluster_count: 2,
            page_offset: 1,
            page_count: 2,
        }],
        "expected the public VG debug snapshot to expose the first per-instance NodeAndClusterCull root seed worklist so host tooling can inspect the exact root traversal inputs instead of inferring them from the wider extract"
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_dispatch_setup,
        Some(RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
            instance_seed_count: 1,
            cluster_budget: snapshot.cull_input.cluster_budget,
            page_budget: snapshot.cull_input.page_budget,
            workgroup_size: 64,
            dispatch_group_count: [1, 1, 1],
        }),
        "expected the public VG debug snapshot to expose the first explicit NodeAndClusterCull dispatch/setup record so host tooling can inspect how the typed NaniteGlobalStateBuffer-style input is translated into concrete startup work before real compute traversal lands"
    );
    assert!(snapshot
        .execution_segments
        .iter()
        .all(|segment| segment.entity == mesh));
    assert!(snapshot
        .execution_segments
        .iter()
        .enumerate()
        .all(|(expected_index, segment)| segment.original_index as usize == expected_index));
    assert!(snapshot.execution_segments.iter().any(|segment| {
        segment.page_id == 200
            && segment.state == RenderVirtualGeometryExecutionState::PendingUpload
    }));
    assert!(snapshot.execution_segments.iter().any(|segment| {
        segment.page_id == 300 && segment.state == RenderVirtualGeometryExecutionState::Resident
    }));
    assert!(!snapshot.submission_order.is_empty());
    assert!(snapshot
        .submission_order
        .iter()
        .all(|entry| entry.entity == mesh));
    assert!(snapshot
        .submission_order
        .iter()
        .all(|entry| [200, 300].contains(&entry.page_id)));
    assert!(snapshot
        .submission_order
        .iter()
        .all(|entry| entry.instance_index == Some(0)));
    assert!(snapshot
        .submission_records
        .iter()
        .all(|record| record.entity == mesh));
    assert!(snapshot
        .submission_records
        .iter()
        .all(|record| [200, 300].contains(&record.page_id)));
    assert!(snapshot
        .submission_records
        .iter()
        .all(|record| record.instance_index == Some(0)));
    assert!(snapshot
        .submission_records
        .iter()
        .enumerate()
        .all(|(expected_index, record)| record.original_index as usize == expected_index));
    assert!(snapshot
        .submission_records
        .iter()
        .all(|record| record.draw_ref_index.is_some()));
    assert_eq!(
        snapshot
            .leaf_clusters
            .iter()
            .map(|cluster| (cluster.cluster_id, cluster.page_id, cluster.lod_level))
            .collect::<Vec<_>>(),
        vec![(20, 200, 10), (30, 300, 10)]
    );
    assert_eq!(
        snapshot.selected_clusters,
        vec![
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 10,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
            },
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 10,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        ]
    );
    assert_eq!(
        snapshot.visbuffer_debug_marks,
        vec![
            RenderVirtualGeometryVisBufferMark {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                page_id: 200,
                lod_level: 10,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
                color_rgba: [92, 190, 130, 255],
            },
            RenderVirtualGeometryVisBufferMark {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                page_id: 300,
                lod_level: 10,
                state: RenderVirtualGeometryExecutionState::Resident,
                color_rgba: [218, 138, 180, 255],
            },
        ]
    );
    assert_eq!(snapshot.bvh_visualization_instances, Vec::new());

    let mut cleared_extract = world.to_render_frame_extract();
    cleared_extract.apply_viewport_size(viewport_size);
    server
        .submit_frame_extract(viewport, cleared_extract)
        .expect("non-vg submission should clear vg snapshot");

    assert_eq!(
        server
            .query_virtual_geometry_debug_snapshot()
            .expect("snapshot query should succeed"),
        None
    );
}

#[test]
fn render_framework_exposes_virtual_geometry_cpu_reference_bvh_inspection_for_automatic_extract() {
    let root = unique_temp_project_root("vg_debug_snapshot_auto_bvh");
    let paths = ProjectPaths::from_root(&root).expect("project paths should resolve");
    paths
        .ensure_layout()
        .expect("project layout should be created");
    ProjectManifest::new(
        "VirtualGeometryDebugSnapshot",
        AssetUri::parse("res://scenes/main.scene.toml").expect("scene uri should parse"),
        1,
    )
    .save(paths.manifest_path())
    .expect("manifest should save");
    fs::create_dir_all(paths.assets_root().join("models"))
        .expect("model directory should be created");
    fs::create_dir_all(paths.assets_root().join("scenes"))
        .expect("scene directory should be created");
    fs::write(
        paths
            .assets_root()
            .join("models")
            .join("nanite_teapot.model.toml"),
        sample_virtual_geometry_model_asset()
            .to_toml_string()
            .expect("model asset should serialize"),
    )
    .expect("model asset should write");
    fs::write(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        SceneAsset {
            entities: Vec::new(),
        }
        .to_toml_string()
        .expect("scene asset should serialize"),
    )
    .expect("scene asset should write");

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .expect("project should open");
    let server =
        WgpuRenderFramework::new(asset_manager.clone()).expect("framework should initialize");
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-debug-snapshot-auto-bvh")
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
                .with_async_compute(false),
        )
        .expect("quality profile should be accepted");

    let model_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("res://models/nanite_teapot.model.toml")
                .expect("model uri should parse"),
        )
        .expect("model resource id should resolve");
    let material_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("builtin://material/default")
                .expect("builtin material uri should parse"),
        )
        .expect("builtin material resource id should resolve");

    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes = vec![
        zircon_runtime::core::framework::render::RenderMeshSnapshot {
            node_id: 101,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(model_id),
            material: ResourceHandle::<MaterialMarker>::new(material_id),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        },
    ];
    snapshot.virtual_geometry_debug = Some(RenderVirtualGeometryDebugState {
        forced_mip: Some(10),
        visualize_bvh: true,
        print_leaf_clusters: true,
        ..RenderVirtualGeometryDebugState::default()
    });

    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(2), snapshot);
    extract.apply_viewport_size(viewport_size);
    server
        .submit_frame_extract(viewport, extract)
        .expect("automatic virtual geometry submission should succeed");

    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot query should succeed")
        .expect("virtual geometry snapshot should be present");
    assert_eq!(snapshot.instances.len(), 1);
    assert_eq!(
        snapshot.cpu_reference_instances,
        vec![RenderVirtualGeometryCpuReferenceInstance {
            instance_index: 0,
            entity: 101,
            mesh_name: Some("NaniteTest".to_string()),
            source_hint: Some("unit-test".to_string()),
            visited_nodes: vec![
                RenderVirtualGeometryCpuReferenceNodeVisit {
                    node_id: 0,
                    depth: 0,
                    page_id: 0,
                    mip_level: 0,
                    is_leaf: false,
                    cluster_ids: Vec::new(),
                },
                RenderVirtualGeometryCpuReferenceNodeVisit {
                    node_id: 1,
                    depth: 1,
                    page_id: 10,
                    mip_level: 10,
                    is_leaf: true,
                    cluster_ids: vec![100, 200],
                },
                RenderVirtualGeometryCpuReferenceNodeVisit {
                    node_id: 2,
                    depth: 1,
                    page_id: 30,
                    mip_level: 10,
                    is_leaf: true,
                    cluster_ids: vec![300],
                },
            ],
            leaf_clusters: vec![
                RenderVirtualGeometryCpuReferenceLeafCluster {
                    node_id: 1,
                    cluster_ordinal: 0,
                    cluster_id: 100,
                    page_id: 10,
                    mip_level: 10,
                    loaded: true,
                    parent_cluster_id: None,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
                RenderVirtualGeometryCpuReferenceLeafCluster {
                    node_id: 1,
                    cluster_ordinal: 1,
                    cluster_id: 200,
                    page_id: 20,
                    mip_level: 9,
                    loaded: false,
                    parent_cluster_id: Some(100),
                    bounds_center: [0.5, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.1,
                },
                RenderVirtualGeometryCpuReferenceLeafCluster {
                    node_id: 2,
                    cluster_ordinal: 2,
                    cluster_id: 300,
                    page_id: 30,
                    mip_level: 10,
                    loaded: true,
                    parent_cluster_id: Some(100),
                    bounds_center: [1.0, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.15,
                },
            ],
            loaded_leaf_clusters: vec![
                RenderVirtualGeometryCpuReferenceLeafCluster {
                    node_id: 1,
                    cluster_ordinal: 0,
                    cluster_id: 100,
                    page_id: 10,
                    mip_level: 10,
                    loaded: true,
                    parent_cluster_id: None,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
                RenderVirtualGeometryCpuReferenceLeafCluster {
                    node_id: 2,
                    cluster_ordinal: 2,
                    cluster_id: 300,
                    page_id: 30,
                    mip_level: 10,
                    loaded: true,
                    parent_cluster_id: Some(100),
                    bounds_center: [1.0, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.15,
                },
            ],
            mip_accepted_clusters: vec![
                RenderVirtualGeometryCpuReferenceLeafCluster {
                    node_id: 1,
                    cluster_ordinal: 0,
                    cluster_id: 100,
                    page_id: 10,
                    mip_level: 10,
                    loaded: true,
                    parent_cluster_id: None,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
                RenderVirtualGeometryCpuReferenceLeafCluster {
                    node_id: 2,
                    cluster_ordinal: 2,
                    cluster_id: 300,
                    page_id: 30,
                    mip_level: 10,
                    loaded: true,
                    parent_cluster_id: Some(100),
                    bounds_center: [1.0, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.15,
                },
            ],
            selected_clusters: vec![
                RenderVirtualGeometryCpuReferenceSelectedCluster {
                    node_id: 1,
                    cluster_ordinal: 0,
                    cluster_id: 100,
                    page_id: 10,
                    mip_level: 10,
                    loaded: true,
                },
                RenderVirtualGeometryCpuReferenceSelectedCluster {
                    node_id: 2,
                    cluster_ordinal: 2,
                    cluster_id: 300,
                    page_id: 30,
                    mip_level: 10,
                    loaded: true,
                },
            ],
            page_cluster_map: vec![
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 10,
                    cluster_ids: vec![100],
                },
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 20,
                    cluster_ids: vec![200],
                },
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 30,
                    cluster_ids: vec![300],
                },
            ],
            loaded_page_cluster_map: vec![
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 10,
                    cluster_ids: vec![100],
                },
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 30,
                    cluster_ids: vec![300],
                },
            ],
            mip_accepted_page_cluster_map: vec![
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 10,
                    cluster_ids: vec![100],
                },
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 30,
                    cluster_ids: vec![300],
                },
            ],
            loaded_mip_cluster_map: vec![RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
                mip_level: 10,
                cluster_ids: vec![100, 300],
            }],
            selected_page_cluster_map: vec![
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 10,
                    cluster_ids: vec![100],
                },
                RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: 30,
                    cluster_ids: vec![300],
                },
            ],
            depth_cluster_map: vec![RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
                depth: 1,
                cluster_ids: vec![100, 200, 300],
            }],
            loaded_depth_cluster_map: vec![RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
                depth: 1,
                cluster_ids: vec![100, 300],
            }],
            mip_accepted_depth_cluster_map: vec![
                RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
                    depth: 1,
                    cluster_ids: vec![100, 300],
                },
            ],
            selected_depth_cluster_map: vec![
                RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
                    depth: 1,
                    cluster_ids: vec![100, 300],
                }
            ],
            mip_cluster_map: vec![
                RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
                    mip_level: 9,
                    cluster_ids: vec![200],
                },
                RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
                    mip_level: 10,
                    cluster_ids: vec![100, 300],
                },
            ],
            selected_mip_cluster_map: vec![RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
                mip_level: 10,
                cluster_ids: vec![100, 300],
            }],
        }]
    );
    assert_eq!(
        snapshot.bvh_visualization_instances,
        vec![RenderVirtualGeometryBvhVisualizationInstance {
            instance_index: 0,
            entity: 101,
            mesh_name: Some("NaniteTest".to_string()),
            source_hint: Some("unit-test".to_string()),
            nodes: vec![
                RenderVirtualGeometryBvhVisualizationNode {
                    node_id: 0,
                    parent_node_id: None,
                    child_node_ids: vec![1, 2],
                    depth: 0,
                    page_id: 0,
                    mip_level: 0,
                    is_leaf: false,
                    cluster_ids: Vec::new(),
                    selected_cluster_ids: vec![100, 300],
                    resident_cluster_ids: vec![100, 300],
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 2.0,
                    screen_space_error: 1.0,
                },
                RenderVirtualGeometryBvhVisualizationNode {
                    node_id: 1,
                    parent_node_id: Some(0),
                    child_node_ids: Vec::new(),
                    depth: 1,
                    page_id: 10,
                    mip_level: 10,
                    is_leaf: true,
                    cluster_ids: vec![100, 200],
                    selected_cluster_ids: vec![100],
                    resident_cluster_ids: vec![100],
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 1.0,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryBvhVisualizationNode {
                    node_id: 2,
                    parent_node_id: Some(0),
                    child_node_ids: Vec::new(),
                    depth: 1,
                    page_id: 30,
                    mip_level: 10,
                    is_leaf: true,
                    cluster_ids: vec![300],
                    selected_cluster_ids: vec![300],
                    resident_cluster_ids: vec![300],
                    bounds_center: [1.0, 0.0, 0.0],
                    bounds_radius: 1.0,
                    screen_space_error: 0.2,
                },
            ],
        }]
    );
    assert_eq!(snapshot.visbuffer_debug_marks, Vec::new());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_framework_automatic_virtual_geometry_bvh_selected_clusters_follow_forced_mip_override() {
    let root = unique_temp_project_root("vg_debug_snapshot_auto_forced_mip");
    let paths = ProjectPaths::from_root(&root).expect("project paths should resolve");
    paths
        .ensure_layout()
        .expect("project layout should be created");
    ProjectManifest::new(
        "VirtualGeometryAutomaticForcedMip",
        AssetUri::parse("res://scenes/main.scene.toml").expect("scene uri should parse"),
        1,
    )
    .save(paths.manifest_path())
    .expect("manifest should save");
    fs::create_dir_all(paths.assets_root().join("models"))
        .expect("model directory should be created");
    fs::create_dir_all(paths.assets_root().join("scenes"))
        .expect("scene directory should be created");
    fs::write(
        paths
            .assets_root()
            .join("models")
            .join("nanite_teapot.model.toml"),
        sample_virtual_geometry_model_asset_with_root_page_table(vec![10, 20, 30])
            .to_toml_string()
            .expect("model asset should serialize"),
    )
    .expect("model asset should write");
    fs::write(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        SceneAsset {
            entities: Vec::new(),
        }
        .to_toml_string()
        .expect("scene asset should serialize"),
    )
    .expect("scene asset should write");

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .expect("project should open");

    let model_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("res://models/nanite_teapot.model.toml")
                .expect("model uri should parse"),
        )
        .expect("model resource id should resolve");
    let material_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("builtin://material/default")
                .expect("builtin material uri should parse"),
        )
        .expect("builtin material resource id should resolve");

    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes = vec![
        zircon_runtime::core::framework::render::RenderMeshSnapshot {
            node_id: 101,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(model_id),
            material: ResourceHandle::<MaterialMarker>::new(material_id),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        },
    ];
    snapshot.virtual_geometry_debug = Some(RenderVirtualGeometryDebugState {
        forced_mip: Some(10),
        visualize_bvh: true,
        ..RenderVirtualGeometryDebugState::default()
    });

    let server = WgpuRenderFramework::new(asset_manager).expect("framework should initialize");
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-debug-snapshot-auto-forced-mip")
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
                .with_async_compute(false),
        )
        .expect("quality profile should be accepted");

    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(3), snapshot);
    extract.apply_viewport_size(viewport_size);
    server
        .submit_frame_extract(viewport, extract)
        .expect("automatic virtual geometry submission should succeed");

    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot query should succeed")
        .expect("virtual geometry snapshot should be present");
    let bvh = snapshot
        .bvh_visualization_instances
        .first()
        .expect("automatic virtual geometry snapshot should publish BVH visualization");
    let root_node = bvh
        .nodes
        .iter()
        .find(|node| node.node_id == 0)
        .expect("root BVH node should exist");
    let left_leaf = bvh
        .nodes
        .iter()
        .find(|node| node.node_id == 1)
        .expect("left leaf BVH node should exist");

    assert_eq!(
        left_leaf.resident_cluster_ids,
        vec![100, 200],
        "expected the forced-mip fixture to keep the mip-9 cluster resident so the test can distinguish residency from selection"
    );
    assert_eq!(
        left_leaf.selected_cluster_ids,
        vec![100],
        "expected automatic BVH visualization to respect forced_mip when computing selected clusters instead of treating every resident cluster as selected"
    );
    assert_eq!(
        root_node.selected_cluster_ids,
        vec![100, 300],
        "expected the automatic root BVH node to exclude resident clusters that fail the forced_mip filter"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_framework_visualize_bvh_changes_captured_frame_for_automatic_virtual_geometry() {
    let root = unique_temp_project_root("vg_debug_snapshot_bvh_overlay");
    let paths = ProjectPaths::from_root(&root).expect("project paths should resolve");
    paths
        .ensure_layout()
        .expect("project layout should be created");
    ProjectManifest::new(
        "VirtualGeometryBvhOverlay",
        AssetUri::parse("res://scenes/main.scene.toml").expect("scene uri should parse"),
        1,
    )
    .save(paths.manifest_path())
    .expect("manifest should save");
    fs::create_dir_all(paths.assets_root().join("models"))
        .expect("model directory should be created");
    fs::create_dir_all(paths.assets_root().join("scenes"))
        .expect("scene directory should be created");
    fs::write(
        paths
            .assets_root()
            .join("models")
            .join("nanite_teapot.model.toml"),
        sample_virtual_geometry_model_asset()
            .to_toml_string()
            .expect("model asset should serialize"),
    )
    .expect("model asset should write");
    fs::write(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        SceneAsset {
            entities: Vec::new(),
        }
        .to_toml_string()
        .expect("scene asset should serialize"),
    )
    .expect("scene asset should write");

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .expect("project should open");

    let model_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("res://models/nanite_teapot.model.toml")
                .expect("model uri should parse"),
        )
        .expect("model resource id should resolve");
    let material_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("builtin://material/default")
                .expect("builtin material uri should parse"),
        )
        .expect("builtin material resource id should resolve");
    let model = ResourceHandle::<ModelMarker>::new(model_id);
    let material = ResourceHandle::<MaterialMarker>::new(material_id);
    let viewport_size = UVec2::new(160, 120);

    let (without_overlay, _) = capture_automatic_virtual_geometry_frame_with_debug(
        asset_manager.clone(),
        model,
        material,
        viewport_size,
        RenderVirtualGeometryDebugState::default(),
    );
    let (with_overlay, snapshot) = capture_automatic_virtual_geometry_frame_with_debug(
        asset_manager.clone(),
        model,
        material,
        viewport_size,
        RenderVirtualGeometryDebugState {
            visualize_bvh: true,
            ..RenderVirtualGeometryDebugState::default()
        },
    );

    let differing_pixels = without_overlay
        .rgba
        .chunks_exact(4)
        .zip(with_overlay.rgba.chunks_exact(4))
        .filter(|(without, with)| without != with)
        .count();
    let highlighted_without = bright_overlay_pixels(&without_overlay.rgba);
    let highlighted_with = bright_overlay_pixels(&with_overlay.rgba);

    assert!(
        !snapshot.bvh_visualization_instances.is_empty(),
        "automatic virtual geometry snapshot should publish drawable BVH instances"
    );
    assert!(
        differing_pixels > 16,
        "expected visualize_bvh to change captured pixels, only {differing_pixels} pixels differed"
    );
    assert!(
        highlighted_with > highlighted_without,
        "expected BVH overlay to add bright debug pixels, without={highlighted_without}, with={highlighted_with}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_framework_visualize_visbuffer_changes_captured_frame_for_automatic_virtual_geometry() {
    let root = unique_temp_project_root("vg_debug_snapshot_visbuffer_overlay");
    let paths = ProjectPaths::from_root(&root).expect("project paths should resolve");
    paths
        .ensure_layout()
        .expect("project layout should be created");
    ProjectManifest::new(
        "VirtualGeometryVisBufferOverlay",
        AssetUri::parse("res://scenes/main.scene.toml").expect("scene uri should parse"),
        1,
    )
    .save(paths.manifest_path())
    .expect("manifest should save");
    fs::create_dir_all(paths.assets_root().join("models"))
        .expect("model directory should be created");
    fs::create_dir_all(paths.assets_root().join("scenes"))
        .expect("scene directory should be created");
    fs::write(
        paths
            .assets_root()
            .join("models")
            .join("nanite_teapot.model.toml"),
        sample_virtual_geometry_model_asset()
            .to_toml_string()
            .expect("model asset should serialize"),
    )
    .expect("model asset should write");
    fs::write(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        SceneAsset {
            entities: Vec::new(),
        }
        .to_toml_string()
        .expect("scene asset should serialize"),
    )
    .expect("scene asset should write");

    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .expect("project should open");

    let model_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("res://models/nanite_teapot.model.toml")
                .expect("model uri should parse"),
        )
        .expect("model resource id should resolve");
    let material_id = asset_manager
        .resolve_asset_id(
            &AssetUri::parse("builtin://material/default")
                .expect("builtin material uri should parse"),
        )
        .expect("builtin material resource id should resolve");
    let model = ResourceHandle::<ModelMarker>::new(model_id);
    let material = ResourceHandle::<MaterialMarker>::new(material_id);
    let viewport_size = UVec2::new(160, 120);

    let (without_overlay, _) = capture_automatic_virtual_geometry_frame_with_debug(
        asset_manager.clone(),
        model,
        material,
        viewport_size,
        RenderVirtualGeometryDebugState::default(),
    );
    let (with_overlay, snapshot) = capture_automatic_virtual_geometry_frame_with_debug(
        asset_manager.clone(),
        model,
        material,
        viewport_size,
        RenderVirtualGeometryDebugState {
            visualize_visbuffer: true,
            ..RenderVirtualGeometryDebugState::default()
        },
    );

    let differing_pixels = without_overlay
        .rgba
        .chunks_exact(4)
        .zip(with_overlay.rgba.chunks_exact(4))
        .filter(|(without, with)| without != with)
        .count();

    assert!(
        !snapshot.visbuffer_debug_marks.is_empty(),
        "automatic virtual geometry snapshot should publish visbuffer debug marks"
    );
    assert!(
        differing_pixels > 8,
        "expected visualize_visbuffer to change captured pixels, only {differing_pixels} pixels differed"
    );

    let _ = fs::remove_dir_all(root);
}

fn virtual_geometry_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        lod_level,
        parent_cluster_id: None,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}

fn virtual_geometry_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_runtime_{label}_{nonce}"))
}

fn sample_virtual_geometry_model_asset() -> ModelAsset {
    sample_virtual_geometry_model_asset_with_root_page_table(vec![10, 30])
}

fn sample_virtual_geometry_model_asset_with_root_page_table(
    root_page_table: Vec<u32>,
) -> ModelAsset {
    let mut virtual_geometry = sample_virtual_geometry_asset();
    virtual_geometry.root_page_table = root_page_table;
    ModelAsset {
        uri: AssetUri::parse("res://models/nanite_teapot.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: vec![
                MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO),
                MeshVertex::new(Vec3::X, Vec3::Y, Vec2::X),
                MeshVertex::new(Vec3::Z, Vec3::Y, Vec2::Y),
            ],
            indices: vec![0, 1, 2],
            virtual_geometry: Some(virtual_geometry),
        }],
    }
}

fn capture_automatic_virtual_geometry_frame_with_debug(
    asset_manager: Arc<ProjectAssetManager>,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    viewport_size: UVec2,
    debug: RenderVirtualGeometryDebugState,
) -> (
    zircon_runtime::core::framework::render::CapturedFrame,
    zircon_runtime::core::framework::render::RenderVirtualGeometryDebugSnapshot,
) {
    let server =
        WgpuRenderFramework::new(asset_manager).expect("framework should initialize for capture");
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-debug-overlay")
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
                .with_async_compute(false),
        )
        .expect("quality profile should be accepted");

    let extract = automatic_virtual_geometry_frame_extract(model, material, viewport_size, debug);
    server
        .submit_frame_extract(viewport, extract)
        .expect("virtual geometry submission should succeed");

    let frame = server
        .capture_frame(viewport)
        .expect("capture should succeed")
        .expect("frame should be available after submission");
    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot query should succeed")
        .expect("virtual geometry snapshot should be present");

    (frame, snapshot)
}

fn automatic_virtual_geometry_frame_extract(
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    viewport_size: UVec2,
    debug: RenderVirtualGeometryDebugState,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.preview.skybox_enabled = false;
    snapshot.preview.lighting_enabled = false;
    snapshot.preview.clear_color = Vec4::ZERO;
    snapshot.scene.directional_lights.clear();
    snapshot.scene.point_lights.clear();
    snapshot.scene.spot_lights.clear();
    snapshot.scene.meshes = vec![RenderMeshSnapshot {
        node_id: 101,
        transform: Transform::default(),
        model,
        material,
        tint: Vec4::new(0.08, 0.08, 0.08, 1.0),
        mobility: Mobility::Dynamic,
        render_layer_mask: default_render_layer_mask(),
    }];
    snapshot.overlays = Default::default();
    snapshot.virtual_geometry_debug = Some(debug);

    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(3), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract
}

fn bright_overlay_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| {
            pixel[3] == 255
                && (pixel[0] > 200 || pixel[1] > 200 || pixel[2] > 200)
                && pixel[0].max(pixel[1]).max(pixel[2]) > 200
        })
        .count()
}

fn sample_virtual_geometry_asset() -> VirtualGeometryAsset {
    VirtualGeometryAsset {
        hierarchy_buffer: vec![
            VirtualGeometryHierarchyNodeAsset {
                node_id: 0,
                parent_node_id: None,
                child_node_ids: vec![1, 2],
                cluster_start: 0,
                cluster_count: 0,
                page_id: 0,
                mip_level: 0,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 2.0,
                screen_space_error: 1.0,
            },
            VirtualGeometryHierarchyNodeAsset {
                node_id: 1,
                parent_node_id: Some(0),
                child_node_ids: Vec::new(),
                cluster_start: 0,
                cluster_count: 2,
                page_id: 10,
                mip_level: 10,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.25,
            },
            VirtualGeometryHierarchyNodeAsset {
                node_id: 2,
                parent_node_id: Some(0),
                child_node_ids: Vec::new(),
                cluster_start: 2,
                cluster_count: 1,
                page_id: 30,
                mip_level: 10,
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.2,
            },
        ],
        cluster_headers: vec![
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 100,
                page_id: 10,
                hierarchy_node_id: 1,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.2,
            },
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 200,
                page_id: 20,
                hierarchy_node_id: 1,
                lod_level: 9,
                parent_cluster_id: Some(100),
                bounds_center: [0.5, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.1,
            },
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 300,
                page_id: 30,
                hierarchy_node_id: 2,
                lod_level: 10,
                parent_cluster_id: Some(100),
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.15,
            },
        ],
        cluster_page_headers: vec![
            VirtualGeometryClusterPageHeaderAsset {
                page_id: 10,
                start_offset: 0,
                payload_size_bytes: 32,
            },
            VirtualGeometryClusterPageHeaderAsset {
                page_id: 20,
                start_offset: 32,
                payload_size_bytes: 32,
            },
            VirtualGeometryClusterPageHeaderAsset {
                page_id: 30,
                start_offset: 64,
                payload_size_bytes: 32,
            },
        ],
        cluster_page_data: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
        root_page_table: vec![10, 30],
        root_cluster_ranges: vec![VirtualGeometryRootClusterRangeAsset {
            node_id: 0,
            cluster_start: 0,
            cluster_count: 3,
        }],
        debug: VirtualGeometryDebugMetadataAsset {
            mesh_name: Some("NaniteTest".to_string()),
            source_hint: Some("unit-test".to_string()),
            notes: vec!["cpu-reference".to_string()],
        },
    }
}
