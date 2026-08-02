use crate::core::framework::render::{
    render_mesh_stable_instance_key, RenderVirtualGeometryExtract, RenderVirtualGeometryInstance,
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
        extract
            .instances
            .iter()
            .filter(|instance| stable_instance_key_for_instance(instance) == stable_instance_key)
            .flat_map(|instance| {
                let start = instance.cluster_offset as usize;
                let end = start.saturating_add(instance.cluster_count as usize);
                extract
                    .clusters
                    .get(start..end)
                    .into_iter()
                    .flatten()
                    .map(|cluster| cluster.cluster_id)
            })
            .collect::<Vec<_>>()
    };
    cluster_ids.sort_unstable();
    cluster_ids.dedup();
    cluster_ids
}

fn stable_instance_key_for_instance(instance: &RenderVirtualGeometryInstance) -> u64 {
    instance.stable_instance_key_or_legacy()
}
