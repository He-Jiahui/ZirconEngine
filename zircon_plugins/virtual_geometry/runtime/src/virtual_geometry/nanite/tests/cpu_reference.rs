use super::*;
use zircon_runtime::asset::VirtualGeometryHierarchyNodeAsset;

#[test]
fn visit_node_records_visit_order_and_cluster_ids() {
    let node = VirtualGeometryHierarchyNodeAsset {
        node_id: 7,
        parent_node_id: Some(1),
        child_node_ids: vec![9, 11],
        cluster_start: 0,
        cluster_count: 2,
        page_id: 44,
        mip_level: 6,
        bounds_center: [0.0, 0.0, 0.0],
        bounds_radius: 1.0,
        screen_space_error: 0.25,
    };
    let clusters = vec![
        VirtualGeometryClusterHeaderAsset {
            cluster_id: 100,
            hierarchy_node_id: 7,
            page_id: 10,
            lod_level: 6,
            parent_cluster_id: None,
            bounds_center: [0.0, 0.0, 0.0],
            bounds_radius: 0.5,
            screen_space_error: 0.1,
        },
        VirtualGeometryClusterHeaderAsset {
            cluster_id: 200,
            hierarchy_node_id: 7,
            page_id: 20,
            lod_level: 5,
            parent_cluster_id: Some(100),
            bounds_center: [1.0, 0.0, 0.0],
            bounds_radius: 0.5,
            screen_space_error: 0.2,
        },
    ];
    let mut traversal = VirtualGeometryCpuReferenceTraversalState::default();

    let is_leaf = visit_node(&mut traversal, &node, 3, &clusters);

    assert!(!is_leaf);
    assert_eq!(
        traversal.visited_nodes,
        vec![VirtualGeometryCpuReferenceNodeVisit {
            node_id: 7,
            depth: 3,
            page_id: 44,
            mip_level: 6,
            is_leaf: false,
            cluster_ids: vec![100, 200],
        }]
    );
}

#[test]
fn store_cluster_keeps_all_leafs_and_selects_only_resident_matching_mip() {
    let mut traversal = VirtualGeometryCpuReferenceTraversalState::default();
    let config = VirtualGeometryCpuReferenceConfig::new(VirtualGeometryDebugConfig::new(
        Some(10),
        false,
        false,
        false,
        false,
    ));
    let resident_pages = [10_u32].into_iter().collect::<BTreeSet<_>>();

    store_cluster(
        &mut traversal,
        77,
        5,
        0,
        &VirtualGeometryClusterHeaderAsset {
            cluster_id: 100,
            hierarchy_node_id: 5,
            page_id: 10,
            lod_level: 10,
            parent_cluster_id: None,
            bounds_center: [0.0, 0.0, 0.0],
            bounds_radius: 0.5,
            screen_space_error: 0.1,
        },
        config,
        &resident_pages,
    );
    store_cluster(
        &mut traversal,
        77,
        5,
        1,
        &VirtualGeometryClusterHeaderAsset {
            cluster_id: 200,
            hierarchy_node_id: 5,
            page_id: 20,
            lod_level: 9,
            parent_cluster_id: Some(100),
            bounds_center: [1.0, 0.0, 0.0],
            bounds_radius: 0.5,
            screen_space_error: 0.2,
        },
        config,
        &resident_pages,
    );

    assert_eq!(
        traversal
            .leaf_clusters
            .iter()
            .map(|cluster| {
                (
                    cluster.cluster_ordinal,
                    cluster.cluster_id,
                    cluster.page_id,
                    cluster.loaded,
                )
            })
            .collect::<Vec<_>>(),
        vec![(0, 100, 10, true), (1, 200, 20, false)]
    );
    assert_eq!(
        traversal
            .selected_clusters
            .iter()
            .map(|cluster| (cluster.cluster_ordinal, cluster.cluster_id))
            .collect::<Vec<_>>(),
        vec![(0, 100)]
    );
}

#[test]
fn to_render_extract_carries_authored_hierarchy_child_ranges() {
    let asset = VirtualGeometryAsset {
        hierarchy_buffer: vec![
            VirtualGeometryHierarchyNodeAsset {
                node_id: 5,
                parent_node_id: None,
                child_node_ids: vec![7, 8],
                cluster_start: 0,
                cluster_count: 0,
                page_id: 10,
                mip_level: 9,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 2.0,
                screen_space_error: 0.5,
            },
            VirtualGeometryHierarchyNodeAsset {
                node_id: 7,
                parent_node_id: Some(5),
                child_node_ids: Vec::new(),
                cluster_start: 0,
                cluster_count: 1,
                page_id: 10,
                mip_level: 10,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.1,
            },
            VirtualGeometryHierarchyNodeAsset {
                node_id: 8,
                parent_node_id: Some(5),
                child_node_ids: Vec::new(),
                cluster_start: 1,
                cluster_count: 1,
                page_id: 20,
                mip_level: 10,
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.1,
            },
        ],
        cluster_headers: vec![
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 100,
                hierarchy_node_id: 7,
                page_id: 10,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.1,
            },
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 200,
                hierarchy_node_id: 8,
                page_id: 20,
                lod_level: 10,
                parent_cluster_id: Some(100),
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.1,
            },
        ],
        root_page_table: vec![10, 20],
        ..VirtualGeometryAsset::default()
    };

    let frame =
        VirtualGeometryCpuReferenceFrame::from_asset(77, &asset, &[10, 20], Default::default());
    let extract = frame.to_render_extract(2, 2);

    assert_eq!(
        extract.hierarchy_nodes,
        vec![
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 5,
                child_base: 0,
                child_count: 2,
                cluster_start: 0,
                cluster_count: 0,
            },
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 7,
                child_base: 0,
                child_count: 0,
                cluster_start: 0,
                cluster_count: 1,
            },
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 8,
                child_base: 0,
                child_count: 0,
                cluster_start: 1,
                cluster_count: 1,
            },
        ],
        "expected the CPU reference render extract to carry authored hierarchy child ranges so NodeAndClusterCull can replace fixed fanout without reopening the cooked asset"
    );
    assert_eq!(extract.hierarchy_child_ids, vec![7, 8]);
}

#[test]
fn to_render_extract_flattens_non_contiguous_hierarchy_child_ids() {
    let asset = VirtualGeometryAsset {
        hierarchy_buffer: vec![
            VirtualGeometryHierarchyNodeAsset {
                node_id: 5,
                parent_node_id: None,
                child_node_ids: vec![7, 42],
                cluster_start: 0,
                cluster_count: 0,
                page_id: 10,
                mip_level: 9,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 2.0,
                screen_space_error: 0.5,
            },
            VirtualGeometryHierarchyNodeAsset {
                node_id: 7,
                parent_node_id: Some(5),
                child_node_ids: Vec::new(),
                cluster_start: 0,
                cluster_count: 1,
                page_id: 10,
                mip_level: 10,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.1,
            },
            VirtualGeometryHierarchyNodeAsset {
                node_id: 42,
                parent_node_id: Some(5),
                child_node_ids: Vec::new(),
                cluster_start: 1,
                cluster_count: 1,
                page_id: 20,
                mip_level: 10,
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.1,
            },
        ],
        cluster_headers: vec![
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 100,
                hierarchy_node_id: 7,
                page_id: 10,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.1,
            },
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 200,
                hierarchy_node_id: 42,
                page_id: 20,
                lod_level: 10,
                parent_cluster_id: Some(100),
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.1,
            },
        ],
        root_page_table: vec![10, 20],
        ..VirtualGeometryAsset::default()
    };

    let frame =
        VirtualGeometryCpuReferenceFrame::from_asset(77, &asset, &[10, 20], Default::default());
    let extract = frame.to_render_extract(2, 2);

    assert_eq!(
        extract.hierarchy_child_ids,
        vec![7, 42],
        "expected render extract to preserve authored child ids in a flat table instead of assuming child node ids are contiguous"
    );
    assert_eq!(extract.hierarchy_nodes[0].child_base, 0);
    assert_eq!(extract.hierarchy_nodes[0].child_count, 2);
}
