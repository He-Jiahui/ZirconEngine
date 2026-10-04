use crate::core::framework::render::RenderFrameExtract;

// 在提取阶段按稳定实例键建立确定的处理次序，后续批次、历史快照与视图索引共享此顺序。
pub(crate) fn visibility_mesh_indices(extract: &RenderFrameExtract) -> Vec<usize> {
    let mut indices = (0..extract.geometry.meshes.len()).collect::<Vec<_>>();
    sort_visibility_indices_by_stable_key(&mut indices, |index| {
        extract.geometry.meshes[index].stable_instance_key
    });
    indices
}

fn sort_visibility_indices_by_stable_key(indices: &mut [usize], stable_key: impl Fn(usize) -> u64) {
    indices.sort_unstable_by_key(|index| (stable_key(*index), *index));
}

#[cfg(test)]
#[path = "tests/visibility_entries.rs"]
mod tests;
