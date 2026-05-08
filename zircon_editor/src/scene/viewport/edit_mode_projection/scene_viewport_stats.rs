use zircon_runtime::scene::EntityId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneViewportStats {
    pub(crate) selected_entity: Option<EntityId>,
    pub(crate) node_count: usize,
    pub(crate) visible_node_count: usize,
    pub(crate) camera_count: usize,
    pub(crate) mesh_count: usize,
    pub(crate) light_count: usize,
}
