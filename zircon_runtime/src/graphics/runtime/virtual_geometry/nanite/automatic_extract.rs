use std::collections::{BTreeMap, BTreeSet};

use crate::asset::{ModelAsset, VirtualGeometryAsset};
use crate::core::framework::render::{
    RenderMeshSnapshot, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryBvhVisualizationNode, RenderVirtualGeometryCluster,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryCpuReferenceLeafCluster,
    RenderVirtualGeometryCpuReferenceNodeVisit,
    RenderVirtualGeometryCpuReferencePageClusterMapEntry, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::{Transform, Vec3};
use crate::core::resource::ResourceId;

use super::cpu_reference::{
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryCpuReferenceLeafCluster,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualGeometryAutomaticExtractInstance {
    pub(crate) entity: EntityId,
    pub(crate) source_model: Option<ResourceId>,
    pub(crate) transform: Transform,
    pub(crate) asset: VirtualGeometryAsset,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualGeometryAutomaticExtractOutput {
    pub(crate) extract: RenderVirtualGeometryExtract,
    pub(crate) cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    pub(crate) bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
}

#[cfg(test)]
pub(crate) fn resolve_virtual_geometry_extract(
    enabled: bool,
    authored: Option<RenderVirtualGeometryExtract>,
    automatic_instances: &[VirtualGeometryAutomaticExtractInstance],
) -> Option<RenderVirtualGeometryExtract> {
    if authored.is_some() || !enabled {
        return authored;
    }

    build_virtual_geometry_automatic_extract(automatic_instances).map(|output| output.extract)
}

pub(crate) fn build_virtual_geometry_automatic_extract_from_meshes<F>(
    meshes: &[RenderMeshSnapshot],
    mut load_model: F,
) -> Option<VirtualGeometryAutomaticExtractOutput>
where
    F: FnMut(ResourceId) -> Option<ModelAsset>,
{
    let mut automatic_instances = Vec::new();

    for mesh in meshes {
        let Some(model) = load_model(mesh.model.id()) else {
            continue;
        };
        for primitive in model.primitives {
            let Some(asset) = primitive.virtual_geometry else {
                continue;
            };
            automatic_instances.push(VirtualGeometryAutomaticExtractInstance {
                entity: mesh.node_id,
                source_model: Some(mesh.model.id()),
                transform: mesh.transform,
                asset,
            });
        }
    }

    build_virtual_geometry_automatic_extract(&automatic_instances)
}

pub(crate) fn build_virtual_geometry_automatic_extract(
    automatic_instances: &[VirtualGeometryAutomaticExtractInstance],
) -> Option<VirtualGeometryAutomaticExtractOutput> {
    let mut clusters = Vec::new();
    let mut pages = Vec::new();
    let mut instances = Vec::new();
    let mut cpu_reference_instances = Vec::new();
    let mut bvh_visualization_instances = Vec::new();
    let mut cluster_budget = 0_u32;
    let mut page_budget = 0_u32;
    let mut next_cluster_id = 1_u32;
    let mut next_page_id = 1_u32;

    for instance in automatic_instances {
        if instance.asset.cluster_headers.is_empty()
            && instance.asset.cluster_page_headers.is_empty()
        {
            continue;
        }

        let resident_local_pages = initial_resident_page_ids(&instance.asset);
        let resident_page_list = resident_local_pages.iter().copied().collect::<Vec<_>>();
        let cpu_reference = VirtualGeometryCpuReferenceFrame::from_asset(
            instance.entity,
            &instance.asset,
            &resident_page_list,
            VirtualGeometryCpuReferenceConfig::default(),
        );
        let cpu_reference_instance_index =
            u32::try_from(cpu_reference_instances.len()).unwrap_or(u32::MAX);
        cpu_reference_instances.push(render_cpu_reference_instance(
            cpu_reference_instance_index,
            &cpu_reference,
        ));
        bvh_visualization_instances.push(render_bvh_visualization_instance(
            cpu_reference_instance_index,
            &instance.asset,
            &cpu_reference,
        ));
        let root_cluster_count = root_cluster_count(&instance.asset);
        let per_instance_cluster_budget = cpu_reference
            .selected_clusters
            .len()
            .max(root_cluster_count)
            .max(usize::from(!instance.asset.cluster_headers.is_empty()));
        let per_instance_page_budget = resident_local_pages.len().max(usize::from(
            !instance.asset.cluster_page_headers.is_empty()
                || !instance.asset.cluster_headers.is_empty(),
        ));
        cluster_budget = cluster_budget
            .saturating_add(u32::try_from(per_instance_cluster_budget).unwrap_or(u32::MAX));
        page_budget =
            page_budget.saturating_add(u32::try_from(per_instance_page_budget).unwrap_or(u32::MAX));
        let instance_page_offset = u32::try_from(pages.len()).unwrap_or(u32::MAX);
        let instance_cluster_offset = u32::try_from(clusters.len()).unwrap_or(u32::MAX);

        let local_pages = ordered_local_pages(&instance.asset);
        let mut page_remap = BTreeMap::new();
        for (local_page_id, size_bytes) in local_pages {
            let global_page_id = next_page_id;
            next_page_id = next_page_id.saturating_add(1);
            page_remap.insert(local_page_id, global_page_id);
            pages.push(RenderVirtualGeometryPage {
                page_id: global_page_id,
                resident: resident_local_pages.contains(&local_page_id),
                size_bytes,
            });
        }

        let mut cluster_remap = BTreeMap::new();
        for cluster in &instance.asset.cluster_headers {
            let global_cluster_id = next_cluster_id;
            next_cluster_id = next_cluster_id.saturating_add(1);
            cluster_remap.insert(cluster.cluster_id, global_cluster_id);
        }

        let transform_matrix = instance.transform.matrix();
        let bounds_scale = instance.transform.scale.abs().max_element();
        for cluster in &instance.asset.cluster_headers {
            let Some(&global_cluster_id) = cluster_remap.get(&cluster.cluster_id) else {
                continue;
            };
            let page_id = page_remap
                .get(&cluster.page_id)
                .copied()
                .unwrap_or_default();
            let bounds_center =
                transform_matrix.transform_point3(Vec3::from_array(cluster.bounds_center));
            clusters.push(RenderVirtualGeometryCluster {
                entity: instance.entity,
                cluster_id: global_cluster_id,
                page_id,
                lod_level: cluster.lod_level,
                parent_cluster_id: cluster
                    .parent_cluster_id
                    .and_then(|parent_cluster_id| cluster_remap.get(&parent_cluster_id).copied()),
                bounds_center,
                bounds_radius: cluster.bounds_radius * bounds_scale,
                screen_space_error: cluster.screen_space_error,
            });
        }

        let instance_page_count = u32::try_from(pages.len())
            .unwrap_or(u32::MAX)
            .saturating_sub(instance_page_offset);
        let instance_cluster_count = u32::try_from(clusters.len())
            .unwrap_or(u32::MAX)
            .saturating_sub(instance_cluster_offset);
        instances.push(RenderVirtualGeometryInstance {
            entity: instance.entity,
            source_model: instance.source_model,
            transform: instance.transform,
            cluster_offset: instance_cluster_offset,
            cluster_count: instance_cluster_count,
            page_offset: instance_page_offset,
            page_count: instance_page_count,
            mesh_name: instance.asset.debug.mesh_name.clone(),
            source_hint: instance.asset.debug.source_hint.clone(),
        });
    }

    if clusters.is_empty() && pages.is_empty() {
        return None;
    }

    Some(VirtualGeometryAutomaticExtractOutput {
        extract: RenderVirtualGeometryExtract {
            cluster_budget,
            page_budget,
            clusters,
            pages,
            instances,
            debug: RenderVirtualGeometryDebugState::default(),
        },
        cpu_reference_instances,
        bvh_visualization_instances,
    })
}

fn render_bvh_visualization_instance(
    instance_index: u32,
    asset: &VirtualGeometryAsset,
    frame: &VirtualGeometryCpuReferenceFrame,
) -> RenderVirtualGeometryBvhVisualizationInstance {
    let nodes_by_id = asset
        .hierarchy_buffer
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let visited_nodes = frame
        .visited_nodes
        .iter()
        .map(|visit| (visit.node_id, visit))
        .collect::<BTreeMap<_, _>>();
    let selected_cluster_ids = frame
        .selected_clusters
        .iter()
        .map(|cluster| cluster.cluster_id)
        .collect::<BTreeSet<_>>();

    RenderVirtualGeometryBvhVisualizationInstance {
        instance_index,
        entity: frame.entity,
        mesh_name: frame.mesh_name.clone(),
        source_hint: frame.source_hint.clone(),
        nodes: frame
            .visited_nodes
            .iter()
            .filter_map(|visit| {
                let node = nodes_by_id.get(&visit.node_id).copied()?;
                let subtree_leaf_clusters = subtree_leaf_clusters_for_node(
                    node.node_id,
                    &nodes_by_id,
                    &frame.leaf_clusters,
                );
                Some(RenderVirtualGeometryBvhVisualizationNode {
                    node_id: node.node_id,
                    parent_node_id: node.parent_node_id,
                    child_node_ids: node.child_node_ids.clone(),
                    depth: visited_nodes
                        .get(&node.node_id)
                        .map(|visited_node| visited_node.depth)
                        .unwrap_or_default(),
                    page_id: node.page_id,
                    mip_level: node.mip_level,
                    is_leaf: node.child_node_ids.is_empty(),
                    cluster_ids: visit.cluster_ids.clone(),
                    selected_cluster_ids: subtree_leaf_clusters
                        .iter()
                        .filter(|cluster| selected_cluster_ids.contains(&cluster.cluster_id))
                        .map(|cluster| cluster.cluster_id)
                        .collect(),
                    resident_cluster_ids: subtree_leaf_clusters
                        .iter()
                        .filter(|cluster| cluster.loaded)
                        .map(|cluster| cluster.cluster_id)
                        .collect(),
                    bounds_center: node.bounds_center,
                    bounds_radius: node.bounds_radius,
                    screen_space_error: node.screen_space_error,
                })
            })
            .collect(),
    }
}

fn render_cpu_reference_instance(
    instance_index: u32,
    frame: &VirtualGeometryCpuReferenceFrame,
) -> RenderVirtualGeometryCpuReferenceInstance {
    RenderVirtualGeometryCpuReferenceInstance {
        instance_index,
        entity: frame.entity,
        mesh_name: frame.mesh_name.clone(),
        source_hint: frame.source_hint.clone(),
        visited_nodes: frame
            .visited_nodes
            .iter()
            .map(|visit| RenderVirtualGeometryCpuReferenceNodeVisit {
                node_id: visit.node_id,
                depth: visit.depth,
                page_id: visit.page_id,
                mip_level: visit.mip_level,
                is_leaf: visit.is_leaf,
                cluster_ids: visit.cluster_ids.clone(),
            })
            .collect(),
        leaf_clusters: frame
            .leaf_clusters
            .iter()
            .map(|cluster| RenderVirtualGeometryCpuReferenceLeafCluster {
                node_id: cluster.node_id,
                cluster_id: cluster.cluster_id,
                page_id: cluster.page_id,
                mip_level: cluster.mip_level,
                loaded: cluster.loaded,
                parent_cluster_id: cluster.parent_cluster_id,
                bounds_center: cluster.bounds_center,
                bounds_radius: cluster.bounds_radius,
                screen_space_error: cluster.screen_space_error,
            })
            .collect(),
        page_cluster_map: frame
            .page_cluster_map
            .iter()
            .map(
                |(page_id, cluster_ids)| RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: *page_id,
                    cluster_ids: cluster_ids.clone(),
                },
            )
            .collect(),
    }
}

fn subtree_leaf_clusters_for_node<'a>(
    node_id: u32,
    nodes_by_id: &BTreeMap<u32, &'a crate::asset::VirtualGeometryHierarchyNodeAsset>,
    leaf_clusters: &'a [VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<&'a VirtualGeometryCpuReferenceLeafCluster> {
    let mut subtree_node_ids = BTreeSet::new();
    collect_subtree_node_ids(node_id, nodes_by_id, &mut subtree_node_ids);
    leaf_clusters
        .iter()
        .filter(|cluster| subtree_node_ids.contains(&cluster.node_id))
        .collect()
}

fn collect_subtree_node_ids(
    node_id: u32,
    nodes_by_id: &BTreeMap<u32, &crate::asset::VirtualGeometryHierarchyNodeAsset>,
    subtree_node_ids: &mut BTreeSet<u32>,
) {
    if !subtree_node_ids.insert(node_id) {
        return;
    }
    let Some(node) = nodes_by_id.get(&node_id).copied() else {
        return;
    };
    for child_node_id in &node.child_node_ids {
        collect_subtree_node_ids(*child_node_id, nodes_by_id, subtree_node_ids);
    }
}

fn ordered_local_pages(asset: &VirtualGeometryAsset) -> Vec<(u32, u64)> {
    let mut local_pages = Vec::new();
    let mut seen_page_ids = BTreeSet::new();

    for page in &asset.cluster_page_headers {
        if seen_page_ids.insert(page.page_id) {
            local_pages.push((page.page_id, page.payload_size_bytes));
        }
    }

    let mut extra_page_ids = asset
        .cluster_headers
        .iter()
        .map(|cluster| cluster.page_id)
        .chain(asset.root_page_table.iter().copied())
        .filter(|page_id| !seen_page_ids.contains(page_id))
        .collect::<Vec<_>>();
    extra_page_ids.sort_unstable();
    extra_page_ids.dedup();
    for page_id in extra_page_ids {
        local_pages.push((page_id, 0));
    }

    local_pages
}

fn initial_resident_page_ids(asset: &VirtualGeometryAsset) -> BTreeSet<u32> {
    let resident_page_ids = if asset.root_page_table.is_empty() {
        asset
            .cluster_headers
            .iter()
            .filter(|cluster| cluster.parent_cluster_id.is_none())
            .map(|cluster| cluster.page_id)
            .collect::<Vec<_>>()
    } else {
        asset.root_page_table.clone()
    };

    resident_page_ids.into_iter().collect()
}

fn root_cluster_count(asset: &VirtualGeometryAsset) -> usize {
    let root_cluster_count = asset
        .cluster_headers
        .iter()
        .filter(|cluster| cluster.parent_cluster_id.is_none())
        .count();

    if root_cluster_count == 0 && !asset.cluster_headers.is_empty() {
        1
    } else {
        root_cluster_count
    }
}
