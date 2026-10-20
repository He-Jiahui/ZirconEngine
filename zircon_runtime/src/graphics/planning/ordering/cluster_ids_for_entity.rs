use crate::core::framework::render::{
    render_mesh_stable_instance_key, RenderVirtualGeometryExtract, RenderVirtualGeometryInstance,
};

pub(crate) fn cluster_ids_for_stable_instance_key(
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
) -> Vec<u32> {
    let mut cluster_ids = cluster_ids_from_instances(extract, stable_instance_key);
    cluster_ids.extend(
        extract
            .clusters
            .iter()
            .filter(|candidate| {
                render_mesh_stable_instance_key(candidate.entity, 0) == stable_instance_key
                    && !extract
                        .instances
                        .iter()
                        .any(|instance| instance.entity == candidate.entity)
            })
            .map(|candidate| candidate.cluster_id),
    );
    cluster_ids.sort_unstable();
    cluster_ids.dedup();
    cluster_ids
}

fn cluster_ids_from_instances(
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
) -> Vec<u32> {
    let mut cluster_ids = Vec::new();
    for (index, instance) in extract.instances.iter().enumerate() {
        if stable_instance_key_for_instance(instance) != stable_instance_key {
            continue;
        }
        let Ok(index) = u32::try_from(index) else {
            continue;
        };
        let Ok(rows) =
            extract.cluster_row_indices(instance.entity, stable_instance_key, Some(index))
        else {
            continue;
        };
        cluster_ids.extend(rows.into_iter().map(|row| extract.clusters[row].cluster_id));
    }
    cluster_ids
}

fn stable_instance_key_for_instance(instance: &RenderVirtualGeometryInstance) -> u64 {
    instance.stable_instance_key_or_legacy()
}

#[cfg(test)]
#[path = "tests/segmented_extend.rs"]
mod segmented_extend_tests;
