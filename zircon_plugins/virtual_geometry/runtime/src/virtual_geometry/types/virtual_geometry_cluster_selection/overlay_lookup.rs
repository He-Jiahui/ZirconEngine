use std::collections::HashMap;

use zircon_runtime::core::framework::{
    render::{RenderVirtualGeometryCluster, RenderVirtualGeometryExtract},
    scene::EntityId,
};

#[cfg(test)]
#[path = "overlay_lookup/allocation_tests.rs"]
mod allocation_tests;

pub(super) struct OverlayClusterLookup {
    clusters_by_entity: HashMap<EntityId, Vec<RenderVirtualGeometryCluster>>,
    instance_index_by_cluster: HashMap<(EntityId, u32), u32>,
    ordinal_by_cluster: HashMap<(EntityId, u32), u32>,
}

impl OverlayClusterLookup {
    pub(super) fn new(extract: &RenderVirtualGeometryExtract) -> Self {
        let mut clusters_by_entity = HashMap::<EntityId, Vec<RenderVirtualGeometryCluster>>::new();
        let mut instance_index_by_cluster = HashMap::new();

        if extract.instances.is_empty() {
            for cluster in &extract.clusters {
                clusters_by_entity
                    .entry(cluster.entity)
                    .or_default()
                    .push(*cluster);
            }
        } else {
            for (instance_index, instance) in extract.instances.iter().enumerate() {
                let start = instance.cluster_offset as usize;
                let end = start.saturating_add(instance.cluster_count as usize);
                let Some(instance_clusters) = extract.clusters.get(start..end) else {
                    continue;
                };
                let instance_index = u32::try_from(instance_index).unwrap_or(u32::MAX);
                for cluster in instance_clusters {
                    clusters_by_entity
                        .entry(instance.entity)
                        .or_default()
                        .push(*cluster);
                    instance_index_by_cluster
                        .entry((instance.entity, cluster.cluster_id))
                        .or_insert(instance_index);
                }
            }
        }

        let mut ordinal_by_cluster = HashMap::new();
        for (entity, clusters) in &mut clusters_by_entity {
            clusters.sort_by_key(|cluster| cluster.cluster_id);
            clusters.dedup_by_key(|cluster| cluster.cluster_id);
            ordinal_by_cluster.extend(clusters.iter().enumerate().map(|(ordinal, cluster)| {
                (
                    (*entity, cluster.cluster_id),
                    u32::try_from(ordinal).unwrap_or(u32::MAX),
                )
            }));
        }

        Self {
            clusters_by_entity,
            instance_index_by_cluster,
            ordinal_by_cluster,
        }
    }

    pub(super) fn clusters_for_entity(&self, entity: EntityId) -> &[RenderVirtualGeometryCluster] {
        self.clusters_by_entity
            .get(&entity)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(super) fn instance_index(&self, entity: EntityId, cluster_id: u32) -> Option<u32> {
        self.instance_index_by_cluster
            .get(&(entity, cluster_id))
            .copied()
    }

    pub(super) fn cluster_ordinal(&self, entity: EntityId, cluster_id: u32) -> Option<u32> {
        self.ordinal_by_cluster.get(&(entity, cluster_id)).copied()
    }
}
