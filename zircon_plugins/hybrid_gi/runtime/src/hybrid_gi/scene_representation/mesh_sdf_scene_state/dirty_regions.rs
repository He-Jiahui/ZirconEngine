use std::collections::HashSet;

use zircon_runtime::core::framework::render::RenderMeshBounds;

use super::declarations::HybridGiMeshSdfObject;

pub(super) fn changed_object_regions(
    previous: &[HybridGiMeshSdfObject],
    next: &[HybridGiMeshSdfObject],
) -> Vec<RenderMeshBounds> {
    let mut dirty_regions = Vec::new();
    let mut dirty_region_keys = HashSet::new();
    let mut previous_index = 0;
    let mut next_index = 0;
    while previous_index < previous.len() || next_index < next.len() {
        match (previous.get(previous_index), next.get(next_index)) {
            (Some(previous), Some(next))
                if previous.stable_instance_key < next.stable_instance_key =>
            {
                push_unique_region(&mut dirty_regions, &mut dirty_region_keys, previous.bounds);
                previous_index += 1;
            }
            (Some(previous), Some(next))
                if previous.stable_instance_key > next.stable_instance_key =>
            {
                push_unique_region(&mut dirty_regions, &mut dirty_region_keys, next.bounds);
                next_index += 1;
            }
            (Some(previous), Some(next)) => {
                if previous != next {
                    push_unique_region(&mut dirty_regions, &mut dirty_region_keys, previous.bounds);
                    push_unique_region(&mut dirty_regions, &mut dirty_region_keys, next.bounds);
                }
                previous_index += 1;
                next_index += 1;
            }
            (Some(previous), None) => {
                push_unique_region(&mut dirty_regions, &mut dirty_region_keys, previous.bounds);
                previous_index += 1;
            }
            (None, Some(next)) => {
                push_unique_region(&mut dirty_regions, &mut dirty_region_keys, next.bounds);
                next_index += 1;
            }
            (None, None) => break,
        }
    }
    dirty_regions
}

fn push_unique_region(
    regions: &mut Vec<RenderMeshBounds>,
    region_keys: &mut HashSet<[u32; 6]>,
    region: RenderMeshBounds,
) {
    let key = [
        region.min[0].to_bits(),
        region.min[1].to_bits(),
        region.min[2].to_bits(),
        region.max[0].to_bits(),
        region.max[1].to_bits(),
        region.max[2].to_bits(),
    ];
    if region_keys.insert(key) {
        regions.push(region);
    }
}
