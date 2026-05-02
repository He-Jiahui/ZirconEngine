use std::collections::HashSet;

pub(super) struct MeshDrawBuildContext {
    pub(super) selection: HashSet<u64>,
    pub(super) allowed_virtual_geometry_entities: Option<HashSet<u64>>,
}
