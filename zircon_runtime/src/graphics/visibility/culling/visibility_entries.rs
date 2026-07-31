use crate::core::framework::render::RenderFrameExtract;
use crate::core::framework::scene::{EntityId, Mobility};

pub(crate) fn visibility_entries(
    extract: &RenderFrameExtract,
) -> impl Iterator<Item = (EntityId, Mobility)> + '_ {
    let use_mesh_fallback = extract.visibility.renderables.is_empty();

    extract
        .visibility
        .renderables
        .iter()
        .map(|entry| (entry.entity, entry.mobility))
        .chain(
            extract
                .geometry
                .meshes
                .iter()
                .filter(move |_| use_mesh_fallback)
                .map(|mesh| (mesh.node_id, mesh.mobility)),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn visibility_entries_stream_entity_mobility_without_layer_clones() {
        let source = include_str!("visibility_entries.rs");
        let iterator = concat!("impl Iterator<Item = (", "EntityId, Mobility)>");

        assert!(source.contains(iterator));
        assert!(!source.contains(concat!("renderables", ".clone()")));
    }
}
