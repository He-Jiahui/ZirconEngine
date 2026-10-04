use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryPage, RenderVirtualGeometryPageDependency,
};

use super::*;

#[test]
fn cooked_page_dependencies_drive_runtime_page_parent_map() {
    let extract = RenderVirtualGeometryExtract {
        pages: vec![page(10), page(20), page(30)],
        page_dependencies: vec![
            RenderVirtualGeometryPageDependency {
                page_id: 10,
                parent_page_id: None,
                child_page_ids: vec![20],
            },
            RenderVirtualGeometryPageDependency {
                page_id: 20,
                parent_page_id: Some(10),
                child_page_ids: Vec::new(),
            },
            RenderVirtualGeometryPageDependency {
                page_id: 30,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            },
        ],
        ..RenderVirtualGeometryExtract::default()
    };

    assert_eq!(page_parent_pages(&extract), BTreeMap::from([(20, 10)]));
}

#[test]
fn cooked_page_dependencies_can_recover_parent_links_from_child_rows() {
    let extract = RenderVirtualGeometryExtract {
        pages: vec![page(10), page(20)],
        page_dependencies: vec![RenderVirtualGeometryPageDependency {
            page_id: 10,
            parent_page_id: None,
            child_page_ids: vec![20],
        }],
        ..RenderVirtualGeometryExtract::default()
    };

    assert_eq!(page_parent_pages(&extract), BTreeMap::from([(20, 10)]));
}

#[test]
fn empty_cooked_page_dependency_graph_suppresses_cluster_lineage_fallback() {
    let extract = RenderVirtualGeometryExtract {
        clusters: vec![cluster(1, 10, None), cluster(2, 20, Some(1))],
        pages: vec![page(10), page(20)],
        page_dependencies: vec![RenderVirtualGeometryPageDependency {
            page_id: 10,
            parent_page_id: None,
            child_page_ids: Vec::new(),
        }],
        ..RenderVirtualGeometryExtract::default()
    };

    assert_eq!(page_parent_pages(&extract), BTreeMap::new());
}

#[test]
fn cluster_lineage_remains_fallback_when_cooked_page_dependencies_are_absent() {
    let extract = RenderVirtualGeometryExtract {
        clusters: vec![
            cluster(1, 10, None),
            cluster(2, 20, Some(1)),
            cluster(3, 30, Some(2)),
        ],
        pages: vec![page(10), page(20), page(30)],
        ..RenderVirtualGeometryExtract::default()
    };

    assert_eq!(
        page_parent_pages(&extract),
        BTreeMap::from([(20, 10), (30, 20)])
    );
}

fn page(page_id: u32) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident: false,
        size_bytes: 1,
    }
}

fn cluster(
    cluster_id: u32,
    page_id: u32,
    parent_cluster_id: Option<u32>,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        cluster_id,
        page_id,
        parent_cluster_id,
        ..RenderVirtualGeometryCluster::default()
    }
}
