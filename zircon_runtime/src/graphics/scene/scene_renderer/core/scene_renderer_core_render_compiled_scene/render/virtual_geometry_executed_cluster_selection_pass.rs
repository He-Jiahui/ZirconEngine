mod seed_backed_compat;

use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryExtract, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource,
};
use crate::graphics::scene::scene_renderer::mesh::MeshDraw;
use crate::graphics::types::{
    VirtualGeometryClusterSelection, VirtualGeometryNodeAndClusterCullClusterWorkItem,
};
use wgpu::util::DeviceExt;

use super::virtual_geometry_node_and_cluster_cull_pass::VirtualGeometryNodeAndClusterCullPassOutput;
use seed_backed_compat::{
    collect_execution_cluster_selection_collection_from_root_seeds,
    SeedBackedExecutionSelectionRecord,
};

#[derive(Default)]
pub(super) struct VirtualGeometryExecutedClusterSelectionPassOutput {
    pub(super) selections: Vec<VirtualGeometryClusterSelection>,
    pub(super) selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub(super) source: RenderVirtualGeometrySelectedClusterSource,
    pub(super) selected_cluster_count: u32,
    pub(super) selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
}

#[derive(Default)]
struct ExecutedClusterSelectionCollection {
    selections: Vec<VirtualGeometryClusterSelection>,
    selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
}

impl ExecutedClusterSelectionCollection {
    fn from_selections(selections: Vec<VirtualGeometryClusterSelection>) -> Self {
        let selected_clusters = selections
            .iter()
            .copied()
            .map(VirtualGeometryClusterSelection::to_selected_cluster)
            .collect();
        Self {
            selections,
            selected_clusters,
        }
    }

    fn from_seed_backed_records(records: Vec<SeedBackedExecutionSelectionRecord>) -> Self {
        let (selections, selected_clusters) = records
            .into_iter()
            .map(|record| (record.selection, record.selected_cluster))
            .unzip();
        Self {
            selections,
            selected_clusters,
        }
    }
}

pub(super) fn execute_virtual_geometry_executed_cluster_selection_pass(
    device: &wgpu::Device,
    selected_cluster_pass_enabled: bool,
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    indirect_execution_draws: &[&MeshDraw],
    extract: Option<&RenderVirtualGeometryExtract>,
    node_and_cluster_cull_pass: &VirtualGeometryNodeAndClusterCullPassOutput,
) -> VirtualGeometryExecutedClusterSelectionPassOutput {
    if !selected_cluster_pass_enabled {
        return VirtualGeometryExecutedClusterSelectionPassOutput {
            selections: Vec::new(),
            selected_clusters: Vec::new(),
            source: RenderVirtualGeometrySelectedClusterSource::Unavailable,
            selected_cluster_count: 0,
            selected_cluster_buffer: None,
        };
    }

    let executed_submission_keys = indirect_execution_draws
        .iter()
        .filter_map(|draw| {
            let detail = draw.virtual_geometry_submission_detail?;
            Some((detail.entity, detail.submission_index))
        })
        .collect::<HashSet<_>>();
    let selections = collect_execution_cluster_selections_from_submission_keys(
        cluster_selections,
        &executed_submission_keys,
    );
    let selection_collection = if selections.is_empty() && cluster_selections.is_none() {
        collect_execution_cluster_selection_collection_from_root_seeds(
            extract,
            node_and_cluster_cull_pass,
        )
    } else {
        ExecutedClusterSelectionCollection::from_selections(selections)
    };
    let selected_cluster_count =
        u32::try_from(selection_collection.selected_clusters.len()).unwrap_or(u32::MAX);
    let selected_cluster_buffer =
        create_selected_cluster_buffer(device, &selection_collection.selected_clusters);
    VirtualGeometryExecutedClusterSelectionPassOutput {
        selections: selection_collection.selections,
        selected_clusters: selection_collection.selected_clusters,
        source: if selected_cluster_count == 0 {
            RenderVirtualGeometrySelectedClusterSource::RenderPathClearOnly
        } else {
            RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections
        },
        selected_cluster_count,
        selected_cluster_buffer,
    }
}

fn collect_execution_cluster_selections_from_submission_keys(
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    executed_submission_keys: &HashSet<(u64, u32)>,
) -> Vec<VirtualGeometryClusterSelection> {
    let Some(cluster_selections) = cluster_selections else {
        return Vec::new();
    };
    if executed_submission_keys.is_empty() {
        return Vec::new();
    }

    let mut emitted_clusters = HashSet::<(u64, u32)>::new();
    let mut executed_selections = cluster_selections
        .iter()
        .copied()
        .filter(|selection| {
            executed_submission_keys.contains(&(selection.entity, selection.submission_index))
        })
        .filter(|selection| emitted_clusters.insert((selection.entity, selection.cluster_id)))
        .collect::<Vec<_>>();
    executed_selections.sort_by_key(|selection| {
        (
            selection.instance_index.unwrap_or(u32::MAX),
            selection.entity,
            selection.cluster_ordinal,
            selection.cluster_id,
            selection.page_id,
            selection.lod_level,
            selection.submission_index,
        )
    });
    executed_selections
}

fn create_selected_cluster_buffer(
    device: &wgpu::Device,
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Option<Arc<wgpu::Buffer>> {
    if selected_clusters.is_empty() {
        return None;
    }

    let packed_words = selected_clusters
        .iter()
        .flat_map(RenderVirtualGeometrySelectedCluster::packed_words)
        .collect::<Vec<_>>();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-executed-selected-clusters"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

#[cfg(test)]
mod tests;
