use zircon_runtime::asset::{
    AssetUri, ModelAsset, ModelPrimitiveAsset, VirtualGeometryAsset,
    VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryPageDependencyAsset, VirtualGeometryRootClusterRangeAsset,
};
use zircon_runtime::core::framework::render::RenderCapabilitySummary;
use zircon_runtime::core::framework::render::RenderMeshSnapshot;
use zircon_runtime::core::framework::render::RenderVirtualGeometryCpuReferencePageDependencyEntry;
use zircon_runtime::core::framework::render::RenderVirtualGeometryDebugState;
use zircon_runtime::core::framework::render::RenderVirtualGeometryExtract;
use zircon_runtime::core::framework::render::RenderVirtualGeometryPageDependency;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::scene::components::Mobility;

use crate::virtual_geometry::{
    build_virtual_geometry_automatic_extract, build_virtual_geometry_automatic_extract_from_meshes,
    build_virtual_geometry_automatic_extract_from_meshes_with_debug,
    resolve_virtual_geometry_extract, VirtualGeometryAutomaticExtractInstance,
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryDebugConfig, VirtualGeometryExecutionMode,
};

#[test]
fn virtual_geometry_nanite_execution_mode_picks_flagship_baseline_and_cpu_paths() {
    let flagship = RenderCapabilitySummary {
        virtual_geometry_supported: true,
        supports_offscreen: true,
        ..RenderCapabilitySummary::default()
    };
    let baseline = RenderCapabilitySummary {
        virtual_geometry_supported: false,
        supports_offscreen: true,
        supports_surface: true,
        ..RenderCapabilitySummary::default()
    };
    let cpu = RenderCapabilitySummary {
        virtual_geometry_supported: false,
        supports_offscreen: false,
        supports_surface: false,
        ..RenderCapabilitySummary::default()
    };

    assert_eq!(
        VirtualGeometryExecutionMode::from_capabilities(&flagship),
        VirtualGeometryExecutionMode::FlagshipGpu
    );
    assert_eq!(
        VirtualGeometryExecutionMode::from_capabilities(&baseline),
        VirtualGeometryExecutionMode::BaselineGpu
    );
    assert_eq!(
        VirtualGeometryExecutionMode::from_capabilities(&cpu),
        VirtualGeometryExecutionMode::CpuDebug
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_traverses_hierarchy_maps_pages_and_filters_forced_mip() {
    let asset = sample_virtual_geometry_asset();
    let frame = VirtualGeometryCpuReferenceFrame::from_asset(
        42,
        &asset,
        &[10, 30],
        VirtualGeometryCpuReferenceConfig::new(VirtualGeometryDebugConfig::new(
            Some(10),
            false,
            false,
            false,
            true,
        )),
    );

    assert_eq!(
        frame
            .visited_nodes()
            .iter()
            .map(|visit| visit.node_id())
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        frame
            .leaf_clusters()
            .iter()
            .map(|cluster| (
                cluster.cluster_ordinal(),
                cluster.cluster_id(),
                cluster.page_id(),
                cluster.mip_level(),
                cluster.loaded()
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 100, 10, 10, true),
            (1, 200, 20, 9, false),
            (2, 300, 30, 10, true)
        ]
    );
    assert_eq!(
        frame
            .selected_clusters()
            .iter()
            .map(|cluster| (cluster.cluster_ordinal(), cluster.cluster_id()))
            .collect::<Vec<_>>(),
        vec![(0, 100), (2, 300)]
    );
    assert_eq!(frame.page_cluster_map().get(&10).cloned(), Some(vec![100]));
    assert_eq!(frame.page_cluster_map().get(&20).cloned(), Some(vec![200]));
    assert_eq!(frame.page_cluster_map().get(&30).cloned(), Some(vec![300]));
    assert_eq!(
        frame
            .page_dependencies()
            .iter()
            .map(|(page_id, (parent_page_id, child_page_ids))| {
                (*page_id, *parent_page_id, child_page_ids.clone())
            })
            .collect::<Vec<_>>(),
        vec![
            (10, None, vec![20]),
            (20, Some(10), Vec::new()),
            (30, None, Vec::new()),
        ],
        "expected CPU reference traversal to expose cooked page dependency metadata beside page-to-cluster grouping"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_bridges_selected_clusters_into_render_extract() {
    let asset = sample_virtual_geometry_asset();
    let frame = VirtualGeometryCpuReferenceFrame::from_asset(
        7,
        &asset,
        &[10, 30],
        VirtualGeometryCpuReferenceConfig::new(VirtualGeometryDebugConfig::new(
            Some(10),
            false,
            false,
            false,
            false,
        )),
    );

    let extract = frame.to_render_extract(2, 3);

    assert_eq!(extract.cluster_budget, 2);
    assert_eq!(extract.page_budget, 3);
    assert_eq!(
        extract
            .clusters
            .iter()
            .map(|cluster| (
                cluster.entity,
                cluster.cluster_id,
                cluster.page_id,
                cluster.lod_level
            ))
            .collect::<Vec<_>>(),
        vec![(7, 100, 10, 10), (7, 300, 30, 10)]
    );
    assert_eq!(
        extract
            .pages
            .iter()
            .map(|page| (page.page_id, page.resident, page.size_bytes))
            .collect::<Vec<_>>(),
        vec![(10, true, 32), (20, false, 32), (30, true, 32)]
    );
    assert_eq!(extract.instances.len(), 1);
    assert_eq!(extract.instances[0].entity, 7);
    assert_eq!(extract.instances[0].cluster_count, 2);
    assert_eq!(extract.instances[0].page_count, 3);
    assert_eq!(
        extract.instances[0].mesh_name.as_deref(),
        Some("NaniteTest")
    );
    assert_eq!(extract.debug.forced_mip, Some(10));
}

#[test]
fn virtual_geometry_nanite_automatic_extract_remaps_multiple_instances_and_preserves_world_space_lineage(
) {
    let output = build_virtual_geometry_automatic_extract(&[
        VirtualGeometryAutomaticExtractInstance::new(
            11,
            None,
            Transform::default(),
            sample_virtual_geometry_asset(),
        ),
        VirtualGeometryAutomaticExtractInstance::new(
            22,
            None,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0))
                .with_scale(Vec3::new(2.0, 1.0, 1.0)),
            sample_virtual_geometry_asset(),
        ),
    ])
    .expect("automatic extract should be synthesized from cooked instances");
    let extract = output.extract();

    assert_eq!(extract.cluster_budget, 4);
    assert_eq!(extract.page_budget, 4);
    assert_eq!(
        extract
            .clusters
            .iter()
            .map(|cluster| (
                cluster.entity,
                cluster.cluster_id,
                cluster.page_id,
                cluster.parent_cluster_id,
                cluster.bounds_center.to_array(),
                cluster.bounds_radius,
            ))
            .collect::<Vec<_>>(),
        vec![
            (11, 1, 1, None, [0.0, 0.0, 0.0], 0.5),
            (11, 2, 2, Some(1), [0.5, 0.0, 0.0], 0.5),
            (11, 3, 3, Some(1), [1.0, 0.0, 0.0], 0.5),
            (22, 4, 4, None, [10.0, 0.0, 0.0], 1.0),
            (22, 5, 5, Some(4), [11.0, 0.0, 0.0], 1.0),
            (22, 6, 6, Some(4), [12.0, 0.0, 0.0], 1.0),
        ]
    );
    assert_eq!(
        extract
            .pages
            .iter()
            .map(|page| (page.page_id, page.resident, page.size_bytes))
            .collect::<Vec<_>>(),
        vec![
            (1, true, 32),
            (2, false, 32),
            (3, true, 32),
            (4, true, 32),
            (5, false, 32),
            (6, true, 32),
        ]
    );
    assert_eq!(
        extract.page_dependencies,
        vec![
            RenderVirtualGeometryPageDependency {
                page_id: 1,
                parent_page_id: None,
                child_page_ids: vec![2],
            },
            RenderVirtualGeometryPageDependency {
                page_id: 2,
                parent_page_id: Some(1),
                child_page_ids: Vec::new(),
            },
            RenderVirtualGeometryPageDependency {
                page_id: 3,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            },
            RenderVirtualGeometryPageDependency {
                page_id: 4,
                parent_page_id: None,
                child_page_ids: vec![5],
            },
            RenderVirtualGeometryPageDependency {
                page_id: 5,
                parent_page_id: Some(4),
                child_page_ids: Vec::new(),
            },
            RenderVirtualGeometryPageDependency {
                page_id: 6,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            },
        ],
        "automatic extract should promote cooked page dependencies into global render ids"
    );
    assert_eq!(extract.instances.len(), 2);
    assert_eq!(extract.instances[0].entity, 11);
    assert_eq!(extract.instances[0].cluster_offset, 0);
    assert_eq!(extract.instances[0].cluster_count, 3);
    assert_eq!(extract.instances[0].page_offset, 0);
    assert_eq!(extract.instances[0].page_count, 3);
    assert_eq!(extract.instances[1].entity, 22);
    assert_eq!(extract.instances[1].cluster_offset, 3);
    assert_eq!(extract.instances[1].cluster_count, 3);
    assert_eq!(extract.instances[1].page_offset, 3);
    assert_eq!(extract.instances[1].page_count, 3);
    assert_eq!(extract.debug, Default::default());
    assert_eq!(output.cpu_reference_instances().len(), 2);
    assert_eq!(output.cpu_reference_instances()[0].instance_index, 0);
    assert_eq!(output.cpu_reference_instances()[0].entity, 11);
    assert_eq!(output.cpu_reference_instances()[1].instance_index, 1);
    assert_eq!(output.cpu_reference_instances()[1].entity, 22);
}

#[test]
fn virtual_geometry_nanite_extract_resolution_respects_explicit_payload_and_feature_gate() {
    let authored = RenderVirtualGeometryExtract {
        cluster_budget: 9,
        page_budget: 7,
        clusters: Vec::new(),
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    };
    let instances = vec![VirtualGeometryAutomaticExtractInstance::new(
        99,
        None,
        Transform::default(),
        sample_virtual_geometry_asset(),
    )];

    assert_eq!(
        resolve_virtual_geometry_extract(false, Some(authored.clone()), &instances),
        Some(authored.clone()),
        "disabled fallback must preserve explicitly authored payload"
    );
    assert_eq!(
        resolve_virtual_geometry_extract(false, None, &instances),
        None,
        "disabled fallback must not synthesize Nanite payload"
    );

    let synthesized = resolve_virtual_geometry_extract(true, None, &instances)
        .expect("enabled fallback should synthesize automatic extract");
    assert_eq!(synthesized.cluster_budget, 2);
    assert_eq!(synthesized.page_budget, 2);
    assert_eq!(synthesized.clusters.len(), 3);
    assert_eq!(synthesized.pages.len(), 3);
    assert_eq!(synthesized.instances.len(), 1);
    assert_eq!(synthesized.instances[0].entity, 99);
    assert_eq!(synthesized.debug, Default::default());

    assert_eq!(
        resolve_virtual_geometry_extract(true, Some(authored.clone()), &instances),
        Some(authored),
        "explicit payload must win over automatic fallback when both are available"
    );
}

#[test]
fn virtual_geometry_nanite_mesh_based_automatic_extract_only_collects_cooked_models() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let plain_model_id = ResourceId::from_stable_label("res://models/plain.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes(
        &[
            RenderMeshSnapshot {
                node_id: 5,
                transform: Transform::default(),
                model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
                material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                    "res://materials/cooked.material.toml",
                )),
                tint: Default::default(),
                mobility: Mobility::Dynamic,
                render_layer_mask: 1,
            },
            RenderMeshSnapshot {
                node_id: 6,
                transform: Transform::default(),
                model: ResourceHandle::<ModelMarker>::new(plain_model_id),
                material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                    "res://materials/plain.material.toml",
                )),
                tint: Default::default(),
                mobility: Mobility::Dynamic,
                render_layer_mask: 1,
            },
        ],
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            id if id == plain_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/plain.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: None,
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");
    let extract = output.extract();

    assert_eq!(extract.clusters.len(), 3);
    assert_eq!(extract.pages.len(), 3);
    assert_eq!(
        extract.page_dependencies,
        vec![
            RenderVirtualGeometryPageDependency {
                page_id: 1,
                parent_page_id: None,
                child_page_ids: vec![2],
            },
            RenderVirtualGeometryPageDependency {
                page_id: 2,
                parent_page_id: Some(1),
                child_page_ids: Vec::new(),
            },
            RenderVirtualGeometryPageDependency {
                page_id: 3,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            },
        ]
    );
    assert!(extract.clusters.iter().all(|cluster| cluster.entity == 5));
    assert_eq!(extract.instances.len(), 1);
    assert_eq!(extract.instances[0].entity, 5);
    assert_eq!(extract.instances[0].source_model, Some(cooked_model_id));
    assert_eq!(extract.instances[0].cluster_count, 3);
    assert_eq!(extract.instances[0].page_count, 3);
    assert_eq!(output.cpu_reference_instances().len(), 1);
    assert_eq!(output.cpu_reference_instances()[0].entity, 5);
    assert_eq!(
        output.cpu_reference_instances()[0].page_dependencies,
        vec![
            RenderVirtualGeometryCpuReferencePageDependencyEntry {
                page_id: 10,
                parent_page_id: None,
                child_page_ids: vec![20],
            },
            RenderVirtualGeometryCpuReferencePageDependencyEntry {
                page_id: 20,
                parent_page_id: Some(10),
                child_page_ids: Vec::new(),
            },
            RenderVirtualGeometryCpuReferencePageDependencyEntry {
                page_id: 30,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            },
        ],
        "expected automatic extract debug output to carry the cooked page dependency tree without asking runtime to know plugin state"
    );
}

#[test]
fn virtual_geometry_nanite_mesh_based_automatic_extract_with_debug_keeps_extract_debug_in_sync() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let debug = RenderVirtualGeometryDebugState {
        forced_mip: Some(10),
        visualize_bvh: true,
        print_leaf_clusters: true,
        ..Default::default()
    };
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        debug,
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert_eq!(
        output.extract().debug, debug,
        "expected the automatic extract helper to keep its public extract debug state aligned with the debug config that already drives CPU-reference and BVH synthesis"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_clusters_grouped_by_bvh_depth() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert_eq!(
        output.cpu_reference_instances()[0]
            .depth_cluster_map
            .iter()
            .map(|entry| (entry.depth, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(1, vec![100, 200, 300])],
        "expected the CPU-reference inspection surface to expose cluster ids grouped by BVH depth so the host can dump per-layer Nanite cluster traversal directly"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_leaf_clusters_grouped_by_mip() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert_eq!(
        output.cpu_reference_instances()[0]
            .mip_cluster_map
            .iter()
            .map(|entry| (entry.mip_level, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(9, vec![200]), (10, vec![100, 300])],
        "expected the CPU-reference inspection surface to expose leaf clusters grouped by mip level so the host can print full mip distributions and filter Mip=10 directly"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_selected_clusters_as_worklist() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(10),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert_eq!(
        output.cpu_reference_instances()[0]
            .selected_clusters
            .iter()
            .map(|cluster| (
                cluster.cluster_ordinal,
                cluster.cluster_id,
                cluster.page_id,
                cluster.mip_level,
                cluster.loaded
            ))
            .collect::<Vec<_>>(),
        vec![(0, 100, 10, 10, true), (2, 300, 30, 10, true)],
        "expected the CPU-reference inspection surface to expose the post-selection worklist directly so host tools can consume Nanite teaching output without replaying residency and forced-mip filtering"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_loaded_leaf_clusters_as_worklist() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(9),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert!(
        output.cpu_reference_instances()[0].selected_clusters.is_empty(),
        "forced_mip=9 should reject the loaded mip-10 leafs, keeping the selected worklist empty so the loaded-leaf list proves its distinct residency-only semantics"
    );
    assert_eq!(
        output.cpu_reference_instances()[0]
            .loaded_leaf_clusters
            .iter()
            .map(|cluster| (
                cluster.cluster_ordinal,
                cluster.cluster_id,
                cluster.page_id,
                cluster.mip_level,
                cluster.loaded
            ))
            .collect::<Vec<_>>(),
        vec![(0, 100, 10, 10, true), (2, 300, 30, 10, true)],
        "expected the CPU-reference inspection surface to expose loaded leaf clusters directly so host tools can verify resident pages without replaying the leaf residency filter"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_mip_accepted_clusters_as_worklist() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(9),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert!(
        output.cpu_reference_instances()[0].selected_clusters.is_empty(),
        "forced_mip=9 should still leave the selected worklist empty because the only mip-accepted cluster is not resident"
    );
    assert_eq!(
        output.cpu_reference_instances()[0]
            .mip_accepted_clusters
            .iter()
            .map(|cluster| (
                cluster.cluster_ordinal,
                cluster.cluster_id,
                cluster.page_id,
                cluster.mip_level,
                cluster.loaded
            ))
            .collect::<Vec<_>>(),
        vec![(1, 200, 20, 9, false)],
        "expected the CPU-reference inspection surface to expose the mip-accepted worklist before residency gating so host tools can explain why forced_mip selected no resident clusters"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_mip_accepted_page_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(9),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert!(
        output.cpu_reference_instances()[0].selected_clusters.is_empty(),
        "forced_mip=9 should still leave the selected worklist empty because the only mip-accepted cluster is not resident"
    );
    assert_eq!(
        output.cpu_reference_instances()[0]
            .mip_accepted_page_cluster_map
            .iter()
            .map(|entry| (entry.page_id, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(20, vec![200])],
        "expected the CPU-reference inspection surface to expose the mip-accepted page-to-cluster mapping before residency gating so host tools can explain which page the forced mip selector actually chose"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_loaded_page_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(9),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert!(
        output.cpu_reference_instances()[0].selected_clusters.is_empty(),
        "forced_mip=9 should keep the selected worklist empty so the loaded page map proves its residency-only semantics"
    );
    assert_eq!(
        output.cpu_reference_instances()[0]
            .loaded_page_cluster_map
            .iter()
            .map(|entry| (entry.page_id, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(10, vec![100]), (30, vec![300])],
        "expected the CPU-reference inspection surface to expose loaded page-to-cluster mapping directly so host tools can verify resident page contents without regrouping loaded leafs themselves"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_loaded_mip_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(9),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert!(
        output.cpu_reference_instances()[0].selected_clusters.is_empty(),
        "forced_mip=9 should keep the selected worklist empty so the loaded mip map proves its residency-only semantics"
    );
    assert_eq!(
        output.cpu_reference_instances()[0]
            .loaded_mip_cluster_map
            .iter()
            .map(|entry| (entry.mip_level, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(10, vec![100, 300])],
        "expected the CPU-reference inspection surface to expose loaded clusters grouped by mip so host tools can verify resident leaf distributions without regrouping loaded leaf clusters"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_selected_page_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(10),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert_eq!(
        output.cpu_reference_instances()[0]
            .selected_page_cluster_map
            .iter()
            .map(|entry| (entry.page_id, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(10, vec![100]), (30, vec![300])],
        "expected the CPU-reference inspection surface to expose selected page-to-cluster mapping directly so host tools can inspect the post-selection Nanite worklist by page without regrouping selected clusters"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_loaded_depth_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(9),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert!(
        output.cpu_reference_instances()[0].selected_clusters.is_empty(),
        "forced_mip=9 should keep the selected worklist empty so the loaded depth map proves its residency-only semantics"
    );
    assert_eq!(
        output.cpu_reference_instances()[0]
            .loaded_depth_cluster_map
            .iter()
            .map(|entry| (entry.depth, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(1, vec![100, 300])],
        "expected the CPU-reference inspection surface to expose loaded clusters grouped by BVH depth so host tools can verify resident leaf depth distribution without regrouping loaded leaf clusters"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_mip_accepted_depth_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(9),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert!(
        output.cpu_reference_instances()[0].selected_clusters.is_empty(),
        "forced_mip=9 should still leave the selected worklist empty because the only mip-accepted cluster is not resident"
    );
    assert_eq!(
        output.cpu_reference_instances()[0]
            .mip_accepted_depth_cluster_map
            .iter()
            .map(|entry| (entry.depth, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(1, vec![200])],
        "expected the CPU-reference inspection surface to expose the mip-accepted clusters grouped by BVH depth before residency gating so host tools can explain where the forced mip selector currently lands in the hierarchy"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_selected_mip_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(10),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert_eq!(
        output.cpu_reference_instances()[0]
            .selected_mip_cluster_map
            .iter()
            .map(|entry| (entry.mip_level, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(10, vec![100, 300])],
        "expected the CPU-reference inspection surface to expose the selected-cluster worklist grouped by mip so host tools can compare automatic or forced LOD results without regrouping selected clusters themselves"
    );
}

#[test]
fn virtual_geometry_nanite_cpu_reference_instances_expose_selected_depth_cluster_map() {
    let cooked_model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let output = build_virtual_geometry_automatic_extract_from_meshes_with_debug(
        &[RenderMeshSnapshot {
            node_id: 5,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(cooked_model_id),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cooked.material.toml",
            )),
            tint: Default::default(),
            mobility: Mobility::Dynamic,
            render_layer_mask: 1,
        }],
        RenderVirtualGeometryDebugState {
            forced_mip: Some(10),
            print_leaf_clusters: true,
            ..Default::default()
        },
        |model_id| match model_id {
            id if id == cooked_model_id => Some(ModelAsset {
                uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    virtual_geometry: Some(sample_virtual_geometry_asset()),
                }],
            }),
            _ => None,
        },
    )
    .expect("cooked model should synthesize automatic extract");

    assert_eq!(
        output.cpu_reference_instances()[0]
            .selected_depth_cluster_map
            .iter()
            .map(|entry| (entry.depth, entry.cluster_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(1, vec![100, 300])],
        "expected the CPU-reference inspection surface to expose the selected-cluster worklist grouped by BVH depth so host tools can compare current selection against full traversal depth dumps without regrouping selected clusters"
    );
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
        page_dependencies: vec![
            VirtualGeometryPageDependencyAsset {
                page_id: 10,
                parent_page_id: None,
                child_page_ids: vec![20],
            },
            VirtualGeometryPageDependencyAsset {
                page_id: 20,
                parent_page_id: Some(10),
                child_page_ids: Vec::new(),
            },
            VirtualGeometryPageDependencyAsset {
                page_id: 30,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            },
        ],
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
