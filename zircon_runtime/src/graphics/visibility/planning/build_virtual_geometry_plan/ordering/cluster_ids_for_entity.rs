use crate::core::framework::render::{
    RenderVirtualGeometryExtract, RenderVirtualGeometryInstance, render_mesh_stable_instance_key,
};

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn cluster_ids_for_stable_instance_key(
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
) -> Vec<u32> {
    let mut cluster_ids = if extract.instances.is_empty() {
        extract
            .clusters
            .iter()
            .filter(|candidate| {
                render_mesh_stable_instance_key(candidate.entity, 0) == stable_instance_key
            })
            .map(|candidate| candidate.cluster_id)
            .collect::<Vec<_>>()
    } else {
        cluster_ids_from_instances(extract, stable_instance_key)
    };
    cluster_ids.sort_unstable();
    cluster_ids.dedup();
    cluster_ids
}

fn cluster_ids_from_instances(
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
) -> Vec<u32> {
    let mut cluster_ids = Vec::new();
    for instance in &extract.instances {
        if stable_instance_key_for_instance(instance) != stable_instance_key {
            continue;
        }
        let start = instance.cluster_offset as usize;
        let end = start.saturating_add(instance.cluster_count as usize);
        let Some(clusters) = extract.clusters.get(start..end) else {
            continue;
        };
        cluster_ids.extend(clusters.iter().map(|cluster| cluster.cluster_id));
    }
    cluster_ids
}

fn stable_instance_key_for_instance(instance: &RenderVirtualGeometryInstance) -> u64 {
    instance.stable_instance_key_or_legacy()
}

#[cfg(test)]
#[path = "cluster_ids_for_entity/segmented_extend_tests.rs"]
mod segmented_extend_tests;
