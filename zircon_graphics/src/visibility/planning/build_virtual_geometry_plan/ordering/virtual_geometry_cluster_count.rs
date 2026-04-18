use zircon_scene::RenderVirtualGeometryExtract;

pub(in crate::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_count(
    extract: &RenderVirtualGeometryExtract,
    entity: u64,
) -> u32 {
    let mut cluster_ids = extract
        .clusters
        .iter()
        .filter(|candidate| candidate.entity == entity)
        .map(|candidate| candidate.cluster_id)
        .collect::<Vec<_>>();
    cluster_ids.sort_unstable();
    cluster_ids.dedup();
    cluster_ids.len().max(1) as u32
}
