use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::asset::{ModelAsset, VirtualGeometryAsset};
use crate::core::framework::render::{
    RenderMeshSnapshot, RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage, RenderVirtualGeometryPageDependency,
};
use crate::core::math::Vec3;
use crate::core::resource::ResourceId;
use crate::graphics::{
    VirtualGeometryRuntimeExtractOutput, VirtualGeometryRuntimeFeedback,
    VirtualGeometryRuntimePrepareInput, VirtualGeometryRuntimePrepareOutput,
    VirtualGeometryRuntimeProvider, VirtualGeometryRuntimeProviderRegistration,
    VirtualGeometryRuntimeState, VirtualGeometryRuntimeUpdate,
};

#[derive(Debug)]
struct TestVirtualGeometryRuntimeProvider;

impl VirtualGeometryRuntimeProvider for TestVirtualGeometryRuntimeProvider {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
        Box::new(TestVirtualGeometryRuntimeState)
    }

    fn build_extract_from_meshes(
        &self,
        meshes: &[RenderMeshSnapshot],
        debug: Option<RenderVirtualGeometryDebugState>,
        load_model: &mut dyn FnMut(ResourceId) -> Option<ModelAsset>,
    ) -> Option<VirtualGeometryRuntimeExtractOutput> {
        test_virtual_geometry_extract_from_model_meshes(meshes, debug, load_model)
    }
}

#[derive(Debug)]
struct TestVirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState for TestVirtualGeometryRuntimeState {
    fn prepare_frame(
        &mut self,
        _input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput {
        VirtualGeometryRuntimePrepareOutput::default()
    }

    fn update_after_render(
        &mut self,
        _feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate {
        VirtualGeometryRuntimeUpdate::default()
    }
}

pub(super) fn test_virtual_geometry_runtime_provider() -> VirtualGeometryRuntimeProviderRegistration
{
    VirtualGeometryRuntimeProviderRegistration::new(
        "test.virtual-geometry",
        Arc::new(TestVirtualGeometryRuntimeProvider),
    )
}

fn test_virtual_geometry_extract_from_model_meshes(
    meshes: &[RenderMeshSnapshot],
    debug: Option<RenderVirtualGeometryDebugState>,
    load_model: &mut dyn FnMut(ResourceId) -> Option<ModelAsset>,
) -> Option<VirtualGeometryRuntimeExtractOutput> {
    let mut clusters = Vec::new();
    let mut hierarchy_nodes = Vec::new();
    let mut hierarchy_child_ids = Vec::new();
    let mut pages = Vec::new();
    let mut page_dependencies = Vec::new();
    let mut instances = Vec::new();
    let mut next_cluster_id = 1_u32;
    let mut next_page_id = 1_u32;

    for mesh in meshes {
        let Some(model) = load_model(mesh.model.id()) else {
            continue;
        };
        for primitive in model.primitives {
            let Some(asset) = primitive.virtual_geometry else {
                continue;
            };
            append_test_virtual_geometry_asset(
                mesh,
                mesh.model.id(),
                &asset,
                &mut clusters,
                &mut hierarchy_nodes,
                &mut hierarchy_child_ids,
                &mut pages,
                &mut page_dependencies,
                &mut instances,
                &mut next_cluster_id,
                &mut next_page_id,
            );
        }
    }

    if clusters.is_empty() && pages.is_empty() {
        return None;
    }

    Some(VirtualGeometryRuntimeExtractOutput::new(
        RenderVirtualGeometryExtract {
            cluster_budget: saturated_u32_len(clusters.len()),
            page_budget: saturated_u32_len(pages.len()),
            clusters,
            hierarchy_nodes,
            hierarchy_child_ids,
            pages,
            page_dependencies,
            instances,
            debug: debug.unwrap_or_default(),
        },
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

fn append_test_virtual_geometry_asset(
    mesh: &RenderMeshSnapshot,
    source_model: ResourceId,
    asset: &VirtualGeometryAsset,
    clusters: &mut Vec<RenderVirtualGeometryCluster>,
    hierarchy_nodes: &mut Vec<RenderVirtualGeometryHierarchyNode>,
    hierarchy_child_ids: &mut Vec<u32>,
    pages: &mut Vec<RenderVirtualGeometryPage>,
    page_dependencies: &mut Vec<RenderVirtualGeometryPageDependency>,
    instances: &mut Vec<RenderVirtualGeometryInstance>,
    next_cluster_id: &mut u32,
    next_page_id: &mut u32,
) {
    if asset.cluster_headers.is_empty() && asset.cluster_page_headers.is_empty() {
        return;
    }

    let resident_local_pages = test_initial_resident_page_ids(asset);
    let instance_page_offset = saturated_u32_len(pages.len());
    let instance_cluster_offset = saturated_u32_len(clusters.len());
    let instance_index = saturated_u32_len(instances.len());
    let mut page_remap = BTreeMap::new();

    for (local_page_id, size_bytes) in test_ordered_local_pages(asset) {
        let global_page_id = *next_page_id;
        *next_page_id = (*next_page_id).saturating_add(1);
        page_remap.insert(local_page_id, global_page_id);
        pages.push(RenderVirtualGeometryPage {
            page_id: global_page_id,
            resident: resident_local_pages.contains(&local_page_id),
            size_bytes,
        });
    }

    page_dependencies.extend(test_page_dependencies_for_asset(asset, &page_remap));

    let mut cluster_remap = BTreeMap::new();
    for cluster in &asset.cluster_headers {
        let global_cluster_id = *next_cluster_id;
        *next_cluster_id = (*next_cluster_id).saturating_add(1);
        cluster_remap.insert(cluster.cluster_id, global_cluster_id);
    }

    let (asset_hierarchy_nodes, asset_hierarchy_child_ids) = test_hierarchy_for_asset(
        instance_index,
        asset,
        instance_cluster_offset,
        hierarchy_child_ids,
    );
    hierarchy_nodes.extend(asset_hierarchy_nodes);
    hierarchy_child_ids.extend(asset_hierarchy_child_ids);

    let transform_matrix = mesh.transform.matrix();
    let bounds_scale = mesh.transform.scale.abs().max_element();
    for cluster in &asset.cluster_headers {
        let Some(&global_cluster_id) = cluster_remap.get(&cluster.cluster_id) else {
            continue;
        };
        let page_id = page_remap
            .get(&cluster.page_id)
            .copied()
            .unwrap_or_default();
        clusters.push(RenderVirtualGeometryCluster {
            entity: mesh.node_id,
            cluster_id: global_cluster_id,
            hierarchy_node_id: Some(cluster.hierarchy_node_id),
            page_id,
            lod_level: cluster.lod_level,
            parent_cluster_id: cluster
                .parent_cluster_id
                .and_then(|parent_cluster_id| cluster_remap.get(&parent_cluster_id).copied()),
            bounds_center: transform_matrix
                .transform_point3(Vec3::from_array(cluster.bounds_center)),
            bounds_radius: cluster.bounds_radius * bounds_scale,
            screen_space_error: cluster.screen_space_error,
        });
    }

    instances.push(RenderVirtualGeometryInstance {
        entity: mesh.node_id,
        source_model: Some(source_model),
        transform: mesh.transform,
        cluster_offset: instance_cluster_offset,
        cluster_count: saturated_u32_len(clusters.len()).saturating_sub(instance_cluster_offset),
        page_offset: instance_page_offset,
        page_count: saturated_u32_len(pages.len()).saturating_sub(instance_page_offset),
        mesh_name: asset.debug.mesh_name.clone(),
        source_hint: asset.debug.source_hint.clone(),
    });
}

fn test_page_dependencies_for_asset(
    asset: &VirtualGeometryAsset,
    page_remap: &BTreeMap<u32, u32>,
) -> Vec<RenderVirtualGeometryPageDependency> {
    asset
        .page_dependencies
        .iter()
        .filter_map(|dependency| {
            let page_id = page_remap.get(&dependency.page_id).copied()?;
            let parent_page_id = dependency
                .parent_page_id
                .and_then(|parent_page_id| page_remap.get(&parent_page_id).copied());
            let child_page_ids = normalized_remapped_page_ids(
                dependency
                    .child_page_ids
                    .iter()
                    .filter_map(|child_page_id| page_remap.get(child_page_id).copied()),
            );
            Some(RenderVirtualGeometryPageDependency {
                page_id,
                parent_page_id,
                child_page_ids,
            })
        })
        .collect()
}

fn normalized_remapped_page_ids(page_ids: impl IntoIterator<Item = u32>) -> Vec<u32> {
    let mut page_ids = page_ids.into_iter().collect::<Vec<_>>();
    page_ids.sort_unstable();
    page_ids.dedup();
    page_ids
}

fn test_hierarchy_for_asset(
    instance_index: u32,
    asset: &VirtualGeometryAsset,
    cluster_offset: u32,
    existing_child_ids: &[u32],
) -> (Vec<RenderVirtualGeometryHierarchyNode>, Vec<u32>) {
    let child_id_offset = saturated_u32_len(existing_child_ids.len());
    let mut hierarchy_child_ids = Vec::new();
    let hierarchy_nodes = asset
        .hierarchy_buffer
        .iter()
        .map(|node| {
            let child_base = if node.child_node_ids.is_empty() {
                0
            } else {
                child_id_offset.saturating_add(saturated_u32_len(hierarchy_child_ids.len()))
            };
            hierarchy_child_ids.extend(node.child_node_ids.iter().copied());
            RenderVirtualGeometryHierarchyNode {
                instance_index,
                node_id: node.node_id,
                child_base,
                child_count: saturated_u32_len(node.child_node_ids.len()),
                cluster_start: cluster_offset.saturating_add(node.cluster_start),
                cluster_count: node.cluster_count,
            }
        })
        .collect();
    (hierarchy_nodes, hierarchy_child_ids)
}

fn test_ordered_local_pages(asset: &VirtualGeometryAsset) -> Vec<(u32, u64)> {
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

fn test_initial_resident_page_ids(asset: &VirtualGeometryAsset) -> BTreeSet<u32> {
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

fn saturated_u32_len(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}
