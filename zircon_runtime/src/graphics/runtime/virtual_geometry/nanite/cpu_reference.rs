#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::asset::{VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset};
use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::{Transform, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryDebugConfig {
    pub(crate) forced_mip: Option<u8>,
    pub(crate) freeze_cull: bool,
    pub(crate) visualize_bvh: bool,
    pub(crate) visualize_visbuffer: bool,
    pub(crate) print_leaf_clusters: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryCpuReferenceConfig {
    pub(crate) debug: VirtualGeometryDebugConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryCpuReferenceNodeVisit {
    pub(crate) node_id: u32,
    pub(crate) depth: u32,
    pub(crate) page_id: u32,
    pub(crate) mip_level: u8,
    pub(crate) is_leaf: bool,
    pub(crate) cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualGeometryCpuReferenceLeafCluster {
    pub(crate) entity: EntityId,
    pub(crate) node_id: u32,
    pub(crate) cluster_id: u32,
    pub(crate) page_id: u32,
    pub(crate) mip_level: u8,
    pub(crate) loaded: bool,
    pub(crate) parent_cluster_id: Option<u32>,
    pub(crate) bounds_center: [f32; 3],
    pub(crate) bounds_radius: f32,
    pub(crate) screen_space_error: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VirtualGeometryCpuReferenceFrame {
    pub(crate) visited_nodes: Vec<VirtualGeometryCpuReferenceNodeVisit>,
    pub(crate) leaf_clusters: Vec<VirtualGeometryCpuReferenceLeafCluster>,
    pub(crate) selected_clusters: Vec<VirtualGeometryCpuReferenceLeafCluster>,
    pub(crate) page_cluster_map: BTreeMap<u32, Vec<u32>>,
    pub(crate) entity: EntityId,
    pub(crate) debug: VirtualGeometryDebugConfig,
    pub(crate) mesh_name: Option<String>,
    pub(crate) source_hint: Option<String>,
    resident_pages: BTreeSet<u32>,
    page_sizes: BTreeMap<u32, u64>,
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
        let nodes_by_id = asset
            .hierarchy_buffer
            .iter()
            .map(|node| (node.node_id, node))
            .collect::<BTreeMap<_, _>>();
        let mut visited_nodes = Vec::new();
        let mut leaf_clusters = Vec::new();
        let mut selected_clusters = Vec::new();
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
            visited_nodes.push(VirtualGeometryCpuReferenceNodeVisit {
                node_id: node.node_id,
                depth,
                page_id: node.page_id,
                mip_level: node.mip_level,
                is_leaf: child_ids.is_empty(),
                cluster_ids: cluster_headers
                    .iter()
                    .map(|cluster| cluster.cluster_id)
                    .collect(),
            });

            if child_ids.is_empty() {
                for cluster in cluster_headers {
                    let leaf = VirtualGeometryCpuReferenceLeafCluster {
                        entity,
                        node_id: node.node_id,
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
                        selected_clusters.push(leaf.clone());
                    }
                    leaf_clusters.push(leaf);
                }
                continue;
            }

            for child_id in child_ids.into_iter().rev() {
                stack.push((child_id, depth.saturating_add(1)));
            }
        }

        Self {
            visited_nodes,
            leaf_clusters,
            selected_clusters,
            page_cluster_map,
            entity,
            debug: config.debug,
            mesh_name: asset.debug.mesh_name.clone(),
            source_hint: asset.debug.source_hint.clone(),
            resident_pages,
            page_sizes,
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
            pages,
            instances,
            debug: render_debug_state(self.debug),
        }
    }
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

fn root_node_ids(
    asset: &VirtualGeometryAsset,
    nodes_by_id: &BTreeMap<u32, &crate::asset::VirtualGeometryHierarchyNodeAsset>,
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
