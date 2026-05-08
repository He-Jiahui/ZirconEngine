use zircon_runtime::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SceneHierarchyRow {
    pub(crate) entity: EntityId,
    pub(crate) parent: Option<EntityId>,
    pub(crate) depth: u32,
    pub(crate) display_name: String,
    pub(crate) kind: String,
    pub(crate) selected: bool,
    pub(crate) active_in_hierarchy: bool,
    pub(crate) has_children: bool,
}
