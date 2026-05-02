#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::asset::{VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency,
};
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Transform, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryDebugConfig {
    forced_mip: Option<u8>,
    freeze_cull: bool,
    visualize_bvh: bool,
    visualize_visbuffer: bool,
    print_leaf_clusters: bool,
}

impl VirtualGeometryDebugConfig {
    pub(crate) fn new(
        forced_mip: Option<u8>,
        freeze_cull: bool,
        visualize_bvh: bool,
        visualize_visbuffer: bool,
        print_leaf_clusters: bool,
    ) -> Self {
        Self {
            forced_mip,
            freeze_cull,
            visualize_bvh,
            visualize_visbuffer,
            print_leaf_clusters,
        }
    }

    pub(crate) fn forced_mip(&self) -> Option<u8> {
        self.forced_mip
    }

    pub(crate) fn freeze_cull(&self) -> bool {
        self.freeze_cull
    }

    pub(crate) fn visualize_bvh(&self) -> bool {
        self.visualize_bvh
    }

    pub(crate) fn visualize_visbuffer(&self) -> bool {
        self.visualize_visbuffer
    }

    pub(crate) fn print_leaf_clusters(&self) -> bool {
        self.print_leaf_clusters
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryCpuReferenceConfig {
    debug: VirtualGeometryDebugConfig,
}

impl VirtualGeometryCpuReferenceConfig {
    pub(crate) fn new(debug: VirtualGeometryDebugConfig) -> Self {
        Self { debug }
    }

    pub(crate) fn debug(&self) -> VirtualGeometryDebugConfig {
        self.debug
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryCpuReferenceNodeVisit {
    node_id: u32,
    depth: u32,
    page_id: u32,
    mip_level: u8,
    is_leaf: bool,
    cluster_ids: Vec<u32>,
}

impl VirtualGeometryCpuReferenceNodeVisit {
    pub(crate) fn node_id(&self) -> u32 {
        self.node_id
    }

    pub(crate) fn depth(&self) -> u32 {
        self.depth
    }

    pub(crate) fn page_id(&self) -> u32 {
        self.page_id
    }

    pub(crate) fn mip_level(&self) -> u8 {
        self.mip_level
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    pub(crate) fn cluster_ids(&self) -> &[u32] {
        &self.cluster_ids
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualGeometryCpuReferenceLeafCluster {
    entity: EntityId,
    node_id: u32,
    cluster_ordinal: u32,
    cluster_id: u32,
    page_id: u32,
    mip_level: u8,
    loaded: bool,
    parent_cluster_id: Option<u32>,
    bounds_center: [f32; 3],
    bounds_radius: f32,
    screen_space_error: f32,
}

impl VirtualGeometryCpuReferenceLeafCluster {
    pub(crate) fn entity(&self) -> EntityId {
        self.entity
    }

    pub(crate) fn node_id(&self) -> u32 {
        self.node_id
    }

    pub(crate) fn cluster_ordinal(&self) -> u32 {
        self.cluster_ordinal
    }

    pub(crate) fn cluster_id(&self) -> u32 {
        self.cluster_id
    }

    pub(crate) fn page_id(&self) -> u32 {
        self.page_id
    }

    pub(crate) fn mip_level(&self) -> u8 {
        self.mip_level
    }

    pub(crate) fn loaded(&self) -> bool {
        self.loaded
    }

    pub(crate) fn parent_cluster_id(&self) -> Option<u32> {
        self.parent_cluster_id
    }

    pub(crate) fn bounds_center(&self) -> [f32; 3] {
        self.bounds_center
    }

    pub(crate) fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }

    pub(crate) fn screen_space_error(&self) -> f32 {
        self.screen_space_error
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VirtualGeometryCpuReferenceFrame {
    visited_nodes: Vec<VirtualGeometryCpuReferenceNodeVisit>,
    leaf_clusters: Vec<VirtualGeometryCpuReferenceLeafCluster>,
    selected_clusters: Vec<VirtualGeometryCpuReferenceLeafCluster>,
    page_cluster_map: BTreeMap<u32, Vec<u32>>,
    hierarchy_nodes: Vec<RenderVirtualGeometryHierarchyNode>,
    hierarchy_child_ids: Vec<u32>,
    entity: EntityId,
    debug: VirtualGeometryDebugConfig,
    mesh_name: Option<String>,
    source_hint: Option<String>,
    resident_pages: BTreeSet<u32>,
    page_sizes: BTreeMap<u32, u64>,
    page_dependencies: BTreeMap<u32, (Option<u32>, Vec<u32>)>,
}

impl VirtualGeometryCpuReferenceFrame {
    pub(crate) fn visited_nodes(&self) -> &[VirtualGeometryCpuReferenceNodeVisit] {
        &self.visited_nodes
    }

    pub(crate) fn leaf_clusters(&self) -> &[VirtualGeometryCpuReferenceLeafCluster] {
        &self.leaf_clusters
    }

    pub(crate) fn selected_clusters(&self) -> &[VirtualGeometryCpuReferenceLeafCluster] {
        &self.selected_clusters
    }

    pub(crate) fn page_cluster_map(&self) -> &BTreeMap<u32, Vec<u32>> {
        &self.page_cluster_map
    }

    pub(crate) fn entity(&self) -> EntityId {
        self.entity
    }

    pub(crate) fn debug(&self) -> VirtualGeometryDebugConfig {
        self.debug
    }

    pub(crate) fn mesh_name(&self) -> Option<&str> {
        self.mesh_name.as_deref()
    }

    pub(crate) fn source_hint(&self) -> Option<&str> {
        self.source_hint.as_deref()
    }

    pub(crate) fn page_dependencies(&self) -> &BTreeMap<u32, (Option<u32>, Vec<u32>)> {
        &self.page_dependencies
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct VirtualGeometryCpuReferenceTraversalState {
    visited_nodes: Vec<VirtualGeometryCpuReferenceNodeVisit>,
    leaf_clusters: Vec<VirtualGeometryCpuReferenceLeafCluster>,
    selected_clusters: Vec<VirtualGeometryCpuReferenceLeafCluster>,
}

impl VirtualGeometryCpuReferenceFrame {
    pub(crate) fn from_asset(
        entity: EntityId,
        asset: &VirtualGeometryAsset,
        resident_pages: &[u32],
        config: VirtualGeometryCpuReferenceConfig,
    ) -> Self {
        let resident_pages = resident_pages.iter().copied().collect::<BTreeSet<_>>();
        let page_sizes = asset
            .cluster_page_headers
            .iter()
            .map(|page| (page.page_id, page.payload_size_bytes))
            .collect::<BTreeMap<_, _>>();
        let page_cluster_map = build_page_cluster_map(asset);
        let page_dependencies = build_page_dependency_map(asset);
        let nodes_by_id = asset
            .hierarchy_buffer
            .iter()
            .map(|node| (node.node_id, node))
            .collect::<BTreeMap<_, _>>();
        let mut traversal = VirtualGeometryCpuReferenceTraversalState::default();
        let mut stack = root_node_ids(asset, &nodes_by_id)
            .into_iter()
            .rev()
            .map(|node_id| (node_id, 0_u32))
            .collect::<Vec<_>>();

        while let Some((node_id, depth)) = stack.pop() {
            let Some(node) = nodes_by_id.get(&node_id).copied() else {
                continue;
            };
            let child_ids = node.child_node_ids.clone();
            let cluster_headers =
                cluster_headers_for_node(asset, node.cluster_start, node.cluster_count);
            if visit_node(&mut traversal, node, depth, &cluster_headers) {
                for (cluster_index, cluster) in cluster_headers.iter().enumerate() {
                    store_cluster(
                        &mut traversal,
                        entity,
                        node.node_id,
                        node.cluster_start
                            .saturating_add(u32::try_from(cluster_index).unwrap_or(u32::MAX)),
                        cluster,
                        config,
                        &resident_pages,
                    );
                }
                continue;
            }

            for child_id in child_ids.into_iter().rev() {
                stack.push((child_id, depth.saturating_add(1)));
            }
        }

        let (hierarchy_nodes, hierarchy_child_ids) = render_hierarchy_for_asset(0, asset, 0);

        Self {
            visited_nodes: traversal.visited_nodes,
            leaf_clusters: traversal.leaf_clusters,
            selected_clusters: traversal.selected_clusters,
            page_cluster_map,
            hierarchy_nodes,
            hierarchy_child_ids,
            entity,
            debug: config.debug,
            mesh_name: asset.debug.mesh_name.clone(),
            source_hint: asset.debug.source_hint.clone(),
            resident_pages,
            page_sizes,
            page_dependencies,
        }
    }

    pub(crate) fn to_render_extract(
        &self,
        cluster_budget: u32,
        page_budget: u32,
    ) -> RenderVirtualGeometryExtract {
        let clusters = self
            .selected_clusters
            .iter()
            .take(cluster_budget as usize)
            .map(|cluster| RenderVirtualGeometryCluster {
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                hierarchy_node_id: Some(cluster.node_id),
                page_id: cluster.page_id,
                lod_level: cluster.mip_level,
                parent_cluster_id: cluster.parent_cluster_id,
                bounds_center: Vec3::from_array(cluster.bounds_center),
                bounds_radius: cluster.bounds_radius,
                screen_space_error: cluster.screen_space_error,
            })
            .collect::<Vec<_>>();
        let pages = self
            .page_sizes
            .iter()
            .take(page_budget as usize)
            .map(|(page_id, size_bytes)| RenderVirtualGeometryPage {
                page_id: *page_id,
                resident: self.resident_pages.contains(page_id),
                size_bytes: *size_bytes,
            })
            .collect::<Vec<_>>();
        let instances = if clusters.is_empty() && pages.is_empty() {
            Vec::new()
        } else {
            vec![RenderVirtualGeometryInstance {
                entity: self.entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: u32::try_from(clusters.len()).unwrap_or(u32::MAX),
                page_offset: 0,
                page_count: u32::try_from(pages.len()).unwrap_or(u32::MAX),
                mesh_name: self.mesh_name.clone(),
                source_hint: self.source_hint.clone(),
            }]
        };

        RenderVirtualGeometryExtract {
            cluster_budget,
            page_budget,
            clusters,
            hierarchy_nodes: self.hierarchy_nodes.clone(),
            hierarchy_child_ids: self.hierarchy_child_ids.clone(),
            pages,
            page_dependencies: render_extract_page_dependencies(&self.page_dependencies),
            instances,
            debug: render_debug_state(self.debug),
        }
    }
}

fn render_extract_page_dependencies(
    page_dependencies: &BTreeMap<u32, (Option<u32>, Vec<u32>)>,
) -> Vec<RenderVirtualGeometryPageDependency> {
    page_dependencies
        .iter()
        .map(
            |(page_id, (parent_page_id, child_page_ids))| RenderVirtualGeometryPageDependency {
                page_id: *page_id,
                parent_page_id: *parent_page_id,
                child_page_ids: child_page_ids.clone(),
            },
        )
        .collect()
}

fn render_hierarchy_for_asset(
    instance_index: u32,
    asset: &VirtualGeometryAsset,
    cluster_offset: u32,
) -> (Vec<RenderVirtualGeometryHierarchyNode>, Vec<u32>) {
    let mut hierarchy_child_ids = Vec::new();
    let hierarchy_nodes = asset
        .hierarchy_buffer
        .iter()
        .map(|node| {
            let child_base = if node.child_node_ids.is_empty() {
                0
            } else {
                u32::try_from(hierarchy_child_ids.len()).unwrap_or(u32::MAX)
            };
            hierarchy_child_ids.extend(node.child_node_ids.iter().copied());
            RenderVirtualGeometryHierarchyNode {
                instance_index,
                node_id: node.node_id,
                child_base,
                child_count: u32::try_from(node.child_node_ids.len()).unwrap_or(u32::MAX),
                cluster_start: cluster_offset.saturating_add(node.cluster_start),
                cluster_count: node.cluster_count,
            }
        })
        .collect();
    (hierarchy_nodes, hierarchy_child_ids)
}

fn render_debug_state(debug: VirtualGeometryDebugConfig) -> RenderVirtualGeometryDebugState {
    RenderVirtualGeometryDebugState {
        forced_mip: debug.forced_mip,
        freeze_cull: debug.freeze_cull,
        visualize_bvh: debug.visualize_bvh,
        visualize_visbuffer: debug.visualize_visbuffer,
        print_leaf_clusters: debug.print_leaf_clusters,
    }
}

fn visit_node(
    traversal: &mut VirtualGeometryCpuReferenceTraversalState,
    node: &zircon_runtime::asset::VirtualGeometryHierarchyNodeAsset,
    depth: u32,
    cluster_headers: &[VirtualGeometryClusterHeaderAsset],
) -> bool {
    let is_leaf = node.child_node_ids.is_empty();
    traversal
        .visited_nodes
        .push(VirtualGeometryCpuReferenceNodeVisit {
            node_id: node.node_id,
            depth,
            page_id: node.page_id,
            mip_level: node.mip_level,
            is_leaf,
            cluster_ids: cluster_headers
                .iter()
                .map(|cluster| cluster.cluster_id)
                .collect(),
        });
    is_leaf
}

fn store_cluster(
    traversal: &mut VirtualGeometryCpuReferenceTraversalState,
    entity: EntityId,
    node_id: u32,
    cluster_ordinal: u32,
    cluster: &VirtualGeometryClusterHeaderAsset,
    config: VirtualGeometryCpuReferenceConfig,
    resident_pages: &BTreeSet<u32>,
) {
    let leaf = VirtualGeometryCpuReferenceLeafCluster {
        entity,
        node_id,
        cluster_ordinal,
        cluster_id: cluster.cluster_id,
        page_id: cluster.page_id,
        mip_level: cluster.lod_level,
        loaded: resident_pages.contains(&cluster.page_id),
        parent_cluster_id: cluster.parent_cluster_id,
        bounds_center: cluster.bounds_center,
        bounds_radius: cluster.bounds_radius,
        screen_space_error: cluster.screen_space_error,
    };
    let selected = resident_pages.contains(&cluster.page_id)
        && config
            .debug
            .forced_mip
            .map_or(true, |forced_mip| forced_mip == cluster.lod_level);
    if selected {
        traversal.selected_clusters.push(leaf.clone());
    }
    traversal.leaf_clusters.push(leaf);
}

fn root_node_ids(
    asset: &VirtualGeometryAsset,
    nodes_by_id: &BTreeMap<u32, &zircon_runtime::asset::VirtualGeometryHierarchyNodeAsset>,
) -> Vec<u32> {
    let mut root_ids = if asset.root_cluster_ranges.is_empty() {
        nodes_by_id
            .values()
            .filter(|node| node.parent_node_id.is_none())
            .map(|node| node.node_id)
            .collect::<Vec<_>>()
    } else {
        asset
            .root_cluster_ranges
            .iter()
            .map(|range| range.node_id)
            .collect::<Vec<_>>()
    };
    root_ids.sort_unstable();
    root_ids.dedup();
    root_ids
}

fn cluster_headers_for_node(
    asset: &VirtualGeometryAsset,
    cluster_start: u32,
    cluster_count: u32,
) -> Vec<VirtualGeometryClusterHeaderAsset> {
    let start = cluster_start as usize;
    let end = start
        .saturating_add(cluster_count as usize)
        .min(asset.cluster_headers.len());
    asset.cluster_headers[start..end].to_vec()
}

fn build_page_cluster_map(asset: &VirtualGeometryAsset) -> BTreeMap<u32, Vec<u32>> {
    let mut page_cluster_map = BTreeMap::<u32, Vec<u32>>::new();
    for cluster in &asset.cluster_headers {
        page_cluster_map
            .entry(cluster.page_id)
            .or_default()
            .push(cluster.cluster_id);
    }
    page_cluster_map
}

fn build_page_dependency_map(
    asset: &VirtualGeometryAsset,
) -> BTreeMap<u32, (Option<u32>, Vec<u32>)> {
    if !asset.page_dependencies.is_empty() {
        return asset
            .page_dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.page_id,
                    (
                        dependency.parent_page_id,
                        normalized_child_page_ids(&dependency.child_page_ids),
                    ),
                )
            })
            .collect();
    }

    // Older hand-authored fixtures may not carry the cooked page graph yet, so derive a stable
    // parent/child page view from cluster lineage instead of forcing runtime code to reopen assets.
    let mut page_dependencies = known_page_ids(asset)
        .into_iter()
        .map(|page_id| (page_id, (None, Vec::new())))
        .collect::<BTreeMap<_, _>>();
    let clusters_by_id = asset
        .cluster_headers
        .iter()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<BTreeMap<_, _>>();

    for cluster in &asset.cluster_headers {
        let Some(parent_page_id) = nearest_distinct_parent_page(cluster, &clusters_by_id) else {
            continue;
        };
        page_dependencies.entry(cluster.page_id).or_default().0 = Some(parent_page_id);
        page_dependencies
            .entry(parent_page_id)
            .or_default()
            .1
            .push(cluster.page_id);
    }

    for (_, child_page_ids) in page_dependencies.values_mut() {
        *child_page_ids = normalized_child_page_ids(child_page_ids);
    }

    page_dependencies
}

fn known_page_ids(asset: &VirtualGeometryAsset) -> Vec<u32> {
    let mut page_ids = asset
        .cluster_page_headers
        .iter()
        .map(|page| page.page_id)
        .chain(asset.cluster_headers.iter().map(|cluster| cluster.page_id))
        .chain(asset.root_page_table.iter().copied())
        .collect::<Vec<_>>();
    page_ids.sort_unstable();
    page_ids.dedup();
    page_ids
}

fn nearest_distinct_parent_page(
    cluster: &VirtualGeometryClusterHeaderAsset,
    clusters_by_id: &BTreeMap<u32, &VirtualGeometryClusterHeaderAsset>,
) -> Option<u32> {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = BTreeSet::new();

    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        let parent_cluster = clusters_by_id.get(&parent_cluster_id).copied()?;
        if parent_cluster.page_id != cluster.page_id {
            return Some(parent_cluster.page_id);
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }

    None
}

fn normalized_child_page_ids(child_page_ids: &[u32]) -> Vec<u32> {
    let mut child_page_ids = child_page_ids.to_vec();
    child_page_ids.sort_unstable();
    child_page_ids.dedup();
    child_page_ids
}

#[cfg(test)]
mod tests {
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
}
