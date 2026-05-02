use std::collections::HashMap;

use crate::virtual_geometry::types::VirtualGeometryNodeAndClusterCullClusterWorkItem;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
};

#[derive(Clone, Copy)]
pub(crate) struct SeedBackedClusterOrdering {
    cluster_ordinal: u32,
    entity_cluster_total_count: usize,
}

impl SeedBackedClusterOrdering {
    pub(super) fn new(cluster_ordinal: u32, entity_cluster_total_count: usize) -> Self {
        Self {
            cluster_ordinal,
            entity_cluster_total_count,
        }
    }

    pub(super) fn cluster_ordinal(&self) -> u32 {
        self.cluster_ordinal
    }

    pub(super) fn entity_cluster_total_count(&self) -> usize {
        self.entity_cluster_total_count
    }
}

pub(crate) fn seed_backed_cluster_ordering(
    extract: &RenderVirtualGeometryExtract,
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    let mut clusters_by_entity = HashMap::<u64, Vec<_>>::new();

    if extract.instances.is_empty() {
        for cluster in extract.clusters.iter().copied() {
            clusters_by_entity
                .entry(cluster.entity)
                .or_default()
                .push(cluster);
        }
    } else {
        for instance in &extract.instances {
            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            for cluster in extract
                .clusters
                .get(start..end)
                .into_iter()
                .flatten()
                .copied()
            {
                clusters_by_entity
                    .entry(cluster.entity)
                    .or_default()
                    .push(cluster);
            }
        }
    }

    finalize_seed_backed_cluster_ordering(clusters_by_entity)
}

pub(super) fn seed_backed_cluster_ordering_from_cluster_work_items(
    extract: &RenderVirtualGeometryExtract,
    cluster_work_items: &[VirtualGeometryNodeAndClusterCullClusterWorkItem],
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    if cluster_work_items.is_empty() {
        return seed_backed_cluster_ordering(extract);
    }

    let mut clusters_by_entity = HashMap::<u64, Vec<_>>::new();
    for work_item in cluster_work_items {
        let Some(cluster) = extract
            .clusters
            .get(work_item.cluster_array_index as usize)
            .copied()
        else {
            continue;
        };
        if cluster.entity != work_item.entity {
            continue;
        }
        clusters_by_entity
            .entry(cluster.entity)
            .or_default()
            .push(cluster);
    }

    finalize_seed_backed_cluster_ordering(clusters_by_entity)
}

fn finalize_seed_backed_cluster_ordering(
    clusters_by_entity: HashMap<u64, Vec<RenderVirtualGeometryCluster>>,
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    let mut ordering = HashMap::new();
    for (entity, mut clusters) in clusters_by_entity {
        clusters.sort_by_key(|cluster| cluster.cluster_id);
        clusters.dedup_by_key(|cluster| cluster.cluster_id);
        let entity_cluster_total_count = clusters.len().max(1);
        for (cluster_ordinal, cluster) in clusters.into_iter().enumerate() {
            ordering.insert(
                (entity, cluster.cluster_id),
                SeedBackedClusterOrdering::new(
                    u32::try_from(cluster_ordinal).unwrap_or(u32::MAX),
                    entity_cluster_total_count,
                ),
            );
        }
    }

    ordering
}
