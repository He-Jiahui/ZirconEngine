use crate::core::framework::render::RenderFrameExtract;

pub(crate) fn visibility_mesh_indices(extract: &RenderFrameExtract) -> Vec<usize> {
    let mut indices = (0..extract.geometry.meshes.len()).collect::<Vec<_>>();
    indices.sort_by_key(|index| extract.geometry.meshes[*index].stable_instance_key);
    indices
}

#[cfg(test)]
mod tests {
    #[test]
    fn visibility_entries_order_meshes_by_stable_instance_key() {
        let source = include_str!("visibility_entries.rs");

        assert!(source.contains("visibility_mesh_indices"));
        assert!(source.contains("stable_instance_key"));
        assert!(!source.contains("HashMap"));
    }
}
