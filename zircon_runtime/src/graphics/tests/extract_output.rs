use super::VirtualGeometryRuntimeExtractOutput;

#[test]
fn runtime_extract_output_moves_all_parts_without_clone_projection() {
    let (extract, cpu_references, bvh_instances, resident_payloads) =
        VirtualGeometryRuntimeExtractOutput::default().into_parts();

    assert_eq!(extract, Default::default());
    assert!(cpu_references.is_empty());
    assert!(bvh_instances.is_empty());
    assert!(resident_payloads.is_empty());
}

fn sparse_authored_source_domain() -> crate::core::framework::render::RenderVirtualGeometryExtract {
    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
        RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance,
        RenderVirtualGeometryPage,
    };
    let mut clusters = vec![RenderVirtualGeometryCluster::default(); 91];
    for index in [0, 1, 70, 71, 72, 90] {
        clusters[index] = RenderVirtualGeometryCluster {
            entity: 2,
            cluster_id: index as u32 + 20,
            page_id: 200,
            ..Default::default()
        };
    }
    clusters[0].hierarchy_node_id = Some(71); // No descriptor is authored for this normal row.
    clusters[1].hierarchy_node_id = Some(72);
    clusters[8].entity = 3;
    clusters[9].entity = 4;
    clusters[10].entity = 2; // Same entity alone does not make an explicit row active.
    RenderVirtualGeometryExtract {
        clusters,
        instances: vec![RenderVirtualGeometryInstance {
            entity: 2,
            cluster_count: 2,
            page_count: 1,
            ..Default::default()
        }],
        pages: vec![RenderVirtualGeometryPage {
            page_id: 200,
            ..Default::default()
        }],
        hierarchy_nodes: vec![
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 72,
                child_base: 0,
                child_count: 2,
                cluster_start: 1,
                cluster_count: 1,
            },
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 7,
                cluster_start: 70,
                cluster_count: 3,
                ..Default::default()
            },
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 42,
                cluster_start: 90,
                cluster_count: 1,
                ..Default::default()
            },
        ],
        hierarchy_child_ids: vec![7, 42],
        ..Default::default()
    }
}

#[test]
fn authored_active_rows_keep_sparse_child_ids_visibility_and_normal_ordinals_separate() {
    use crate::core::framework::render::render_mesh_stable_instance_key;
    use std::collections::BTreeSet;
    let extract = sparse_authored_source_domain();
    let key = render_mesh_stable_instance_key(2, 0);
    let active = extract
        .checked_active_cluster_rows(&BTreeSet::from([3]))
        .unwrap();
    assert_eq!(
        active,
        vec![
            (None, render_mesh_stable_instance_key(3, 0), 8),
            (Some(0), key, 0),
            (Some(0), key, 1),
            (Some(0), key, 70),
            (Some(0), key, 71),
            (Some(0), key, 72),
            (Some(0), key, 90),
        ]
    );
    assert_eq!(
        extract.cluster_row_indices(2, key, Some(0)).unwrap(),
        vec![0, 1]
    );
    assert_eq!(extract.clusters.len(), 91);
    assert_eq!(extract.hierarchy_child_ids, vec![7, 42]);
    assert!(!active.iter().any(|&(_, _, row)| [9, 10].contains(&row)));
    assert!(extract.cluster_row_indices(2, key + 1, Some(0)).is_err());
    assert!(extract.cluster_row_indices(2, key, Some(1)).is_err());
    let mut dag = extract.clone();
    dag.hierarchy_child_ids.push(55);
    dag.hierarchy_nodes[1].child_base = 2;
    dag.hierarchy_nodes[1].child_count = 1;
    dag.hierarchy_nodes[2].child_base = 2;
    dag.hierarchy_nodes[2].child_count = 1;
    dag.hierarchy_nodes.push(
        crate::core::framework::render::RenderVirtualGeometryHierarchyNode {
            node_id: 55,
            cluster_start: 90,
            cluster_count: 1,
            ..Default::default()
        },
    );
    assert_eq!(
        dag.checked_active_cluster_rows(&BTreeSet::from([3]))
            .unwrap(),
        active
    );
    let mut cycle = extract.clone();
    cycle.hierarchy_nodes[1].child_count = 1; // Child ID7 leads back to node7 itself.
    assert_eq!(
        cycle
            .checked_active_cluster_rows(&BTreeSet::from([3]))
            .unwrap(),
        active
    );
}

#[test]
fn authored_active_rows_reject_invalid_domains_without_partial_admission() {
    use std::collections::BTreeSet;
    let extract = sparse_authored_source_domain();
    let mut invalid = Vec::new();
    let mut bad = extract.clone();
    bad.clusters[0].entity = 4;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.clusters[70].entity = 4;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.instances[0].cluster_offset = u32::MAX;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.instances[0].page_offset = u32::MAX;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.instances[0].page_count = 2;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.hierarchy_nodes[1].instance_index = 1;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.hierarchy_nodes[0].child_base = u32::MAX;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.hierarchy_nodes[0].child_count = 3;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.hierarchy_child_ids[0] = 99;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.hierarchy_nodes[1].cluster_start = u32::MAX;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.hierarchy_nodes[2].cluster_count = 2;
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.hierarchy_nodes.push(bad.hierarchy_nodes[1]);
    invalid.push(bad);
    let mut bad = extract.clone();
    bad.instances.push(
        crate::core::framework::render::RenderVirtualGeometryInstance {
            entity: 2,
            stable_instance_key: (2 << 16) | 1,
            cluster_offset: 70,
            cluster_count: 3,
            page_count: 1,
            ..Default::default()
        },
    );
    invalid.push(bad);
    for (index, bad) in invalid.iter().enumerate() {
        assert!(
            bad.checked_active_cluster_rows(&BTreeSet::new()).is_err(),
            "invalid case {index}"
        );
        assert!(
            VirtualGeometryRuntimeExtractOutput::default()
                .bind_authored_indexed_topology(bad, &BTreeSet::new())
                .is_err(),
            "invalid admission {index}"
        );
    }
    // An active, valid authored domain still requires an actual immutable source.
    assert!(VirtualGeometryRuntimeExtractOutput::default()
        .bind_authored_indexed_topology(&extract, &BTreeSet::new())
        .is_err());
    let padding = crate::core::framework::render::RenderVirtualGeometryExtract {
        clusters: vec![Default::default(); 91],
        ..Default::default()
    };
    assert!(padding
        .checked_active_cluster_rows(&BTreeSet::new())
        .unwrap()
        .is_empty());
    assert!(VirtualGeometryRuntimeExtractOutput::default()
        .bind_authored_indexed_topology(&padding, &BTreeSet::new())
        .is_ok());
}
