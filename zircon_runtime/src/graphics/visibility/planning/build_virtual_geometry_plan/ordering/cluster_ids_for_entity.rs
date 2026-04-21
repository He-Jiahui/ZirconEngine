use crate::core::framework::render::RenderVirtualGeometryExtract;

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn cluster_ids_for_entity(
    extract: &RenderVirtualGeometryExtract,
    entity: u64,
) -> Vec<u32> {
    let mut cluster_ids = if extract.instances.is_empty() {
        extract
            .clusters
            .iter()
            .filter(|candidate| candidate.entity == entity)
            .map(|candidate| candidate.cluster_id)
            .collect::<Vec<_>>()
    } else {
        extract
            .instances
            .iter()
            .filter(|instance| instance.entity == entity)
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
