#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryIndirectStatsStoreParts;
#[cfg(test)]
use super::read_indirect_authority_records::{
    read_virtual_geometry_indirect_authority_records,
    read_virtual_geometry_indirect_execution_authority_records,
};
#[cfg(test)]
use super::read_indirect_execution_indices::read_virtual_geometry_indirect_execution_draw_ref_indices;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_execution_records(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<(u32, u64, u32, u32, u32)> {
    let execution_authority_records =
        read_virtual_geometry_indirect_execution_authority_records(parts);
    if !execution_authority_records.is_empty() {
        return execution_authority_records
            .into_iter()
            .map(|record| record.execution_record())
            .collect();
    }
    let authority_records = read_virtual_geometry_indirect_authority_records(parts);
    if authority_records.is_empty() {
        return Vec::new();
    }
    let authority_by_draw_ref_index = authority_records
        .into_iter()
        .map(|record| (record.draw_ref_index(), record.execution_record()))
        .collect::<std::collections::HashMap<_, _>>();
    read_virtual_geometry_indirect_execution_draw_ref_indices(parts)
        .into_iter()
        .filter_map(|draw_ref_index| authority_by_draw_ref_index.get(&draw_ref_index).copied())
        .collect()
}
