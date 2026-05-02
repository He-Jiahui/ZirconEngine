use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::asset::{ModelAsset, VirtualGeometryAsset};
use zircon_runtime::core::framework::render::{
    RenderMeshSnapshot, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryBvhVisualizationNode, RenderVirtualGeometryCluster,
    RenderVirtualGeometryCpuReferenceDepthClusterMapEntry,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryCpuReferenceLeafCluster,
    RenderVirtualGeometryCpuReferenceMipClusterMapEntry,
    RenderVirtualGeometryCpuReferenceNodeVisit,
    RenderVirtualGeometryCpuReferencePageClusterMapEntry,
    RenderVirtualGeometryCpuReferenceSelectedCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
};
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::resource::ResourceId;

use super::cpu_reference::{
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryCpuReferenceLeafCluster,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualGeometryAutomaticExtractInstance {
    entity: EntityId,
    source_model: Option<ResourceId>,
    transform: Transform,
    asset: VirtualGeometryAsset,
}

impl VirtualGeometryAutomaticExtractInstance {
    pub(crate) fn new(
        entity: EntityId,
        source_model: Option<ResourceId>,
        transform: Transform,
        asset: VirtualGeometryAsset,
    ) -> Self {
        Self {
            entity,
            source_model,
            transform,
            asset,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualGeometryAutomaticExtractOutput {
    extract: RenderVirtualGeometryExtract,
    cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
}

impl VirtualGeometryAutomaticExtractOutput {
    fn new(
        extract: RenderVirtualGeometryExtract,
        cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
        bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    ) -> Self {
        Self {
            extract,
            cpu_reference_instances,
            bvh_visualization_instances,
        }
    }

    pub(crate) fn extract(&self) -> &RenderVirtualGeometryExtract {
        &self.extract
    }

    pub(crate) fn cpu_reference_instances(&self) -> &[RenderVirtualGeometryCpuReferenceInstance] {
        &self.cpu_reference_instances
    }

    pub(crate) fn bvh_visualization_instances(
        &self,
    ) -> &[RenderVirtualGeometryBvhVisualizationInstance] {
        &self.bvh_visualization_instances
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn into_extract(self) -> RenderVirtualGeometryExtract {
        self.extract
    }
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

    build_virtual_geometry_automatic_extract(automatic_instances)
        .map(VirtualGeometryAutomaticExtractOutput::into_extract)
}

#[cfg(test)]
pub(crate) fn build_virtual_geometry_automatic_extract_from_meshes<F>(
    meshes: &[RenderMeshSnapshot],
    load_model: F,
) -> Option<VirtualGeometryAutomaticExtractOutput>
where
    F: FnMut(ResourceId) -> Option<ModelAsset>,
{
    build_virtual_geometry_automatic_extract_from_meshes_with_config(
        meshes,
        VirtualGeometryCpuReferenceConfig::default(),
        load_model,
    )
}

pub(crate) fn build_virtual_geometry_automatic_extract_from_meshes_with_debug<F>(
    meshes: &[RenderMeshSnapshot],
    debug: RenderVirtualGeometryDebugState,
    load_model: F,
) -> Option<VirtualGeometryAutomaticExtractOutput>
where
    F: FnMut(ResourceId) -> Option<ModelAsset>,
{
    build_virtual_geometry_automatic_extract_from_meshes_with_config(
        meshes,
        cpu_reference_config_for_debug(debug),
        load_model,
    )
}

pub(crate) fn build_virtual_geometry_automatic_extract_from_meshes_with_config<F>(
    meshes: &[RenderMeshSnapshot],
    config: VirtualGeometryCpuReferenceConfig,
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
            automatic_instances.push(VirtualGeometryAutomaticExtractInstance::new(
                mesh.node_id,
                Some(mesh.model.id()),
                mesh.transform,
                asset,
            ));
        }
    }

    build_virtual_geometry_automatic_extract_with_config(&automatic_instances, config)
}

#[cfg(test)]
pub(crate) fn build_virtual_geometry_automatic_extract(
    automatic_instances: &[VirtualGeometryAutomaticExtractInstance],
) -> Option<VirtualGeometryAutomaticExtractOutput> {
    build_virtual_geometry_automatic_extract_with_config(
        automatic_instances,
        VirtualGeometryCpuReferenceConfig::default(),
    )
}

fn build_virtual_geometry_automatic_extract_with_config(
    automatic_instances: &[VirtualGeometryAutomaticExtractInstance],
    config: VirtualGeometryCpuReferenceConfig,
) -> Option<VirtualGeometryAutomaticExtractOutput> {
    let extract_debug = render_debug_state_for_cpu_reference_config(config);
    let mut clusters = Vec::new();
    let mut hierarchy_nodes = Vec::new();
    let mut hierarchy_child_ids = Vec::new();
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
            config,
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
            .selected_clusters()
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
        let instance_index = u32::try_from(instances.len()).unwrap_or(u32::MAX);

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
        let (instance_hierarchy_nodes, instance_hierarchy_child_ids) = render_hierarchy_for_asset(
            instance_index,
            &instance.asset,
            instance_cluster_offset,
            u32::try_from(hierarchy_child_ids.len()).unwrap_or(u32::MAX),
        );
        hierarchy_nodes.extend(instance_hierarchy_nodes);
        hierarchy_child_ids.extend(instance_hierarchy_child_ids);
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
                hierarchy_node_id: Some(cluster.hierarchy_node_id),
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

    Some(VirtualGeometryAutomaticExtractOutput::new(
        RenderVirtualGeometryExtract {
            cluster_budget,
            page_budget,
            clusters,
            hierarchy_nodes,
            hierarchy_child_ids,
            pages,
            instances,
            debug: extract_debug,
        },
        cpu_reference_instances,
        bvh_visualization_instances,
    ))
}

fn render_hierarchy_for_asset(
    instance_index: u32,
    asset: &VirtualGeometryAsset,
    cluster_offset: u32,
    child_id_offset: u32,
) -> (Vec<RenderVirtualGeometryHierarchyNode>, Vec<u32>) {
    let mut hierarchy_child_ids = Vec::new();
    let hierarchy_nodes = asset
        .hierarchy_buffer
        .iter()
        .map(|node| {
            let child_base = if node.child_node_ids.is_empty() {
                0
            } else {
                child_id_offset
                    .saturating_add(u32::try_from(hierarchy_child_ids.len()).unwrap_or(u32::MAX))
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

fn cpu_reference_config_for_debug(
    debug: RenderVirtualGeometryDebugState,
) -> VirtualGeometryCpuReferenceConfig {
    VirtualGeometryCpuReferenceConfig::new(super::cpu_reference::VirtualGeometryDebugConfig::new(
        debug.forced_mip,
        debug.freeze_cull,
        debug.visualize_bvh,
        debug.visualize_visbuffer,
        debug.print_leaf_clusters,
    ))
}

fn render_debug_state_for_cpu_reference_config(
    config: VirtualGeometryCpuReferenceConfig,
) -> RenderVirtualGeometryDebugState {
    let debug = config.debug();
    RenderVirtualGeometryDebugState {
        forced_mip: debug.forced_mip(),
        freeze_cull: debug.freeze_cull(),
        visualize_bvh: debug.visualize_bvh(),
        visualize_visbuffer: debug.visualize_visbuffer(),
        print_leaf_clusters: debug.print_leaf_clusters(),
    }
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
        .visited_nodes()
        .iter()
        .map(|visit| (visit.node_id(), visit))
        .collect::<BTreeMap<_, _>>();
    let selected_cluster_ids = frame
        .selected_clusters()
        .iter()
        .map(|cluster| cluster.cluster_id())
        .collect::<BTreeSet<_>>();

    RenderVirtualGeometryBvhVisualizationInstance {
        instance_index,
        entity: frame.entity(),
        mesh_name: frame.mesh_name().map(str::to_owned),
        source_hint: frame.source_hint().map(str::to_owned),
        nodes: frame
            .visited_nodes()
            .iter()
            .filter_map(|visit| {
                let node = nodes_by_id.get(&visit.node_id()).copied()?;
                let subtree_leaf_clusters = subtree_leaf_clusters_for_node(
                    node.node_id,
                    &nodes_by_id,
                    frame.leaf_clusters(),
                );
                Some(RenderVirtualGeometryBvhVisualizationNode {
                    node_id: node.node_id,
                    parent_node_id: node.parent_node_id,
                    child_node_ids: node.child_node_ids.clone(),
                    depth: visited_nodes
                        .get(&node.node_id)
                        .map(|visited_node| visited_node.depth())
                        .unwrap_or_default(),
                    page_id: node.page_id,
                    mip_level: node.mip_level,
                    is_leaf: node.child_node_ids.is_empty(),
                    cluster_ids: visit.cluster_ids().to_vec(),
                    selected_cluster_ids: subtree_leaf_clusters
                        .iter()
                        .filter(|cluster| selected_cluster_ids.contains(&cluster.cluster_id()))
                        .map(|cluster| cluster.cluster_id())
                        .collect(),
                    resident_cluster_ids: subtree_leaf_clusters
                        .iter()
                        .filter(|cluster| cluster.loaded())
                        .map(|cluster| cluster.cluster_id())
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
        entity: frame.entity(),
        mesh_name: frame.mesh_name().map(str::to_owned),
        source_hint: frame.source_hint().map(str::to_owned),
        visited_nodes: frame
            .visited_nodes()
            .iter()
            .map(|visit| RenderVirtualGeometryCpuReferenceNodeVisit {
                node_id: visit.node_id(),
                depth: visit.depth(),
                page_id: visit.page_id(),
                mip_level: visit.mip_level(),
                is_leaf: visit.is_leaf(),
                cluster_ids: visit.cluster_ids().to_vec(),
            })
            .collect(),
        leaf_clusters: frame
            .leaf_clusters()
            .iter()
            .map(render_cpu_reference_leaf_cluster)
            .collect(),
        loaded_leaf_clusters: frame
            .leaf_clusters()
            .iter()
            .filter(|cluster| cluster.loaded())
            .map(render_cpu_reference_leaf_cluster)
            .collect(),
        mip_accepted_clusters: render_mip_accepted_clusters(frame.leaf_clusters(), frame.debug()),
        selected_clusters: frame
            .selected_clusters()
            .iter()
            .map(|cluster| RenderVirtualGeometryCpuReferenceSelectedCluster {
                node_id: cluster.node_id(),
                cluster_ordinal: cluster.cluster_ordinal(),
                cluster_id: cluster.cluster_id(),
                page_id: cluster.page_id(),
                mip_level: cluster.mip_level(),
                loaded: cluster.loaded(),
            })
            .collect(),
        page_cluster_map: frame
            .page_cluster_map()
            .iter()
            .map(
                |(page_id, cluster_ids)| RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                    page_id: *page_id,
                    cluster_ids: cluster_ids.clone(),
                },
            )
            .collect(),
        loaded_page_cluster_map: render_loaded_page_cluster_map(frame.leaf_clusters()),
        mip_accepted_page_cluster_map: render_mip_accepted_page_cluster_map(
            frame.leaf_clusters(),
            frame.debug(),
        ),
        loaded_mip_cluster_map: render_loaded_mip_cluster_map(frame.leaf_clusters()),
        selected_page_cluster_map: render_selected_page_cluster_map(frame.selected_clusters()),
        depth_cluster_map: render_depth_cluster_map(frame.visited_nodes()),
        loaded_depth_cluster_map: render_loaded_depth_cluster_map(
            frame.visited_nodes(),
            frame.leaf_clusters(),
        ),
        mip_accepted_depth_cluster_map: render_mip_accepted_depth_cluster_map(
            frame.visited_nodes(),
            frame.leaf_clusters(),
            frame.debug(),
        ),
        selected_depth_cluster_map: render_selected_depth_cluster_map(
            frame.visited_nodes(),
            frame.selected_clusters(),
        ),
        mip_cluster_map: render_mip_cluster_map(frame.leaf_clusters()),
        selected_mip_cluster_map: render_selected_mip_cluster_map(frame.selected_clusters()),
    }
}

fn render_cpu_reference_leaf_cluster(
    cluster: &VirtualGeometryCpuReferenceLeafCluster,
) -> RenderVirtualGeometryCpuReferenceLeafCluster {
    RenderVirtualGeometryCpuReferenceLeafCluster {
        node_id: cluster.node_id(),
        cluster_ordinal: cluster.cluster_ordinal(),
        cluster_id: cluster.cluster_id(),
        page_id: cluster.page_id(),
        mip_level: cluster.mip_level(),
        loaded: cluster.loaded(),
        parent_cluster_id: cluster.parent_cluster_id(),
        bounds_center: cluster.bounds_center(),
        bounds_radius: cluster.bounds_radius(),
        screen_space_error: cluster.screen_space_error(),
    }
}

fn render_depth_cluster_map(
    visits: &[super::cpu_reference::VirtualGeometryCpuReferenceNodeVisit],
) -> Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry> {
    let mut cluster_ids_by_depth = BTreeMap::<u32, Vec<u32>>::new();
    for visit in visits {
        if visit.cluster_ids().is_empty() {
            continue;
        }
        cluster_ids_by_depth
            .entry(visit.depth())
            .or_default()
            .extend(visit.cluster_ids().iter().copied());
    }

    cluster_ids_by_depth
        .into_iter()
        .map(
            |(depth, cluster_ids)| RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
                depth,
                cluster_ids,
            },
        )
        .collect()
}

fn render_mip_cluster_map(
    leaf_clusters: &[VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry> {
    let mut cluster_ids_by_mip = BTreeMap::<u8, Vec<u32>>::new();
    for cluster in leaf_clusters {
        cluster_ids_by_mip
            .entry(cluster.mip_level())
            .or_default()
            .push(cluster.cluster_id());
    }

    cluster_ids_by_mip
        .into_iter()
        .map(
            |(mip_level, cluster_ids)| RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
                mip_level,
                cluster_ids,
            },
        )
        .collect()
}

fn render_mip_accepted_clusters(
    leaf_clusters: &[VirtualGeometryCpuReferenceLeafCluster],
    debug: super::cpu_reference::VirtualGeometryDebugConfig,
) -> Vec<RenderVirtualGeometryCpuReferenceLeafCluster> {
    leaf_clusters
        .iter()
        .filter(|cluster| {
            debug
                .forced_mip()
                .map_or(true, |forced_mip| cluster.mip_level() == forced_mip)
        })
        .map(render_cpu_reference_leaf_cluster)
        .collect()
}

fn render_loaded_page_cluster_map(
    leaf_clusters: &[VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry> {
    let mut cluster_ids_by_page = BTreeMap::<u32, Vec<u32>>::new();
    for cluster in leaf_clusters.iter().filter(|cluster| cluster.loaded()) {
        cluster_ids_by_page
            .entry(cluster.page_id())
            .or_default()
            .push(cluster.cluster_id());
    }

    cluster_ids_by_page
        .into_iter()
        .map(
            |(page_id, cluster_ids)| RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                page_id,
                cluster_ids,
            },
        )
        .collect()
}

fn render_mip_accepted_page_cluster_map(
    leaf_clusters: &[VirtualGeometryCpuReferenceLeafCluster],
    debug: super::cpu_reference::VirtualGeometryDebugConfig,
) -> Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry> {
    let mut cluster_ids_by_page = BTreeMap::<u32, Vec<u32>>::new();
    for cluster in leaf_clusters.iter().filter(|cluster| {
        debug
            .forced_mip()
            .map_or(true, |forced_mip| cluster.mip_level() == forced_mip)
    }) {
        cluster_ids_by_page
            .entry(cluster.page_id())
            .or_default()
            .push(cluster.cluster_id());
    }

    cluster_ids_by_page
        .into_iter()
        .map(
            |(page_id, cluster_ids)| RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                page_id,
                cluster_ids,
            },
        )
        .collect()
}

fn render_loaded_mip_cluster_map(
    leaf_clusters: &[VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry> {
    let mut cluster_ids_by_mip = BTreeMap::<u8, Vec<u32>>::new();
    for cluster in leaf_clusters.iter().filter(|cluster| cluster.loaded()) {
        cluster_ids_by_mip
            .entry(cluster.mip_level())
            .or_default()
            .push(cluster.cluster_id());
    }

    cluster_ids_by_mip
        .into_iter()
        .map(
            |(mip_level, cluster_ids)| RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
                mip_level,
                cluster_ids,
            },
        )
        .collect()
}

fn render_selected_page_cluster_map(
    selected_clusters: &[super::cpu_reference::VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry> {
    let mut cluster_ids_by_page = BTreeMap::<u32, Vec<u32>>::new();
    for cluster in selected_clusters {
        cluster_ids_by_page
            .entry(cluster.page_id())
            .or_default()
            .push(cluster.cluster_id());
    }

    cluster_ids_by_page
        .into_iter()
        .map(
            |(page_id, cluster_ids)| RenderVirtualGeometryCpuReferencePageClusterMapEntry {
                page_id,
                cluster_ids,
            },
        )
        .collect()
}

fn render_selected_depth_cluster_map(
    visits: &[super::cpu_reference::VirtualGeometryCpuReferenceNodeVisit],
    selected_clusters: &[super::cpu_reference::VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry> {
    render_depth_cluster_map_for_clusters(visits, selected_clusters)
}

fn render_loaded_depth_cluster_map(
    visits: &[super::cpu_reference::VirtualGeometryCpuReferenceNodeVisit],
    leaf_clusters: &[super::cpu_reference::VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry> {
    let loaded_clusters = leaf_clusters
        .iter()
        .filter(|cluster| cluster.loaded())
        .cloned()
        .collect::<Vec<_>>();
    render_depth_cluster_map_for_clusters(visits, &loaded_clusters)
}

fn render_mip_accepted_depth_cluster_map(
    visits: &[super::cpu_reference::VirtualGeometryCpuReferenceNodeVisit],
    leaf_clusters: &[super::cpu_reference::VirtualGeometryCpuReferenceLeafCluster],
    debug: super::cpu_reference::VirtualGeometryDebugConfig,
) -> Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry> {
    let mip_accepted_clusters = leaf_clusters
        .iter()
        .filter(|cluster| {
            debug
                .forced_mip()
                .map_or(true, |forced_mip| cluster.mip_level() == forced_mip)
        })
        .cloned()
        .collect::<Vec<_>>();
    render_depth_cluster_map_for_clusters(visits, &mip_accepted_clusters)
}

fn render_depth_cluster_map_for_clusters(
    visits: &[super::cpu_reference::VirtualGeometryCpuReferenceNodeVisit],
    clusters: &[super::cpu_reference::VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry> {
    let depth_by_node_id = visits
        .iter()
        .map(|visit| (visit.node_id(), visit.depth()))
        .collect::<BTreeMap<_, _>>();
    let mut cluster_ids_by_depth = BTreeMap::<u32, Vec<u32>>::new();
    for cluster in clusters {
        let Some(&depth) = depth_by_node_id.get(&cluster.node_id()) else {
            continue;
        };
        cluster_ids_by_depth
            .entry(depth)
            .or_default()
            .push(cluster.cluster_id());
    }

    cluster_ids_by_depth
        .into_iter()
        .map(
            |(depth, cluster_ids)| RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
                depth,
                cluster_ids,
            },
        )
        .collect()
}

fn render_selected_mip_cluster_map(
    selected_clusters: &[super::cpu_reference::VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry> {
    let mut cluster_ids_by_mip = BTreeMap::<u8, Vec<u32>>::new();
    for cluster in selected_clusters {
        cluster_ids_by_mip
            .entry(cluster.mip_level())
            .or_default()
            .push(cluster.cluster_id());
    }

    cluster_ids_by_mip
        .into_iter()
        .map(
            |(mip_level, cluster_ids)| RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
                mip_level,
                cluster_ids,
            },
        )
        .collect()
}

fn subtree_leaf_clusters_for_node<'a>(
    node_id: u32,
    nodes_by_id: &BTreeMap<u32, &'a zircon_runtime::asset::VirtualGeometryHierarchyNodeAsset>,
    leaf_clusters: &'a [VirtualGeometryCpuReferenceLeafCluster],
) -> Vec<&'a VirtualGeometryCpuReferenceLeafCluster> {
    let mut subtree_node_ids = BTreeSet::new();
    collect_subtree_node_ids(node_id, nodes_by_id, &mut subtree_node_ids);
    leaf_clusters
        .iter()
        .filter(|cluster| subtree_node_ids.contains(&cluster.node_id()))
        .collect()
}

fn collect_subtree_node_ids(
    node_id: u32,
    nodes_by_id: &BTreeMap<u32, &zircon_runtime::asset::VirtualGeometryHierarchyNodeAsset>,
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
