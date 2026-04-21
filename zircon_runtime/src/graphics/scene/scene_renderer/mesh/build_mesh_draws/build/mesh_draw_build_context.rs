use std::collections::{HashMap, HashSet};

use crate::graphics::types::VirtualGeometryClusterRasterDraw;

pub(super) struct MeshDrawBuildContext {
    pub(super) selection: HashSet<u64>,
    pub(super) virtual_geometry_enabled: bool,
    pub(super) allowed_virtual_geometry_entities: Option<HashSet<u64>>,
    pub(super) virtual_geometry_cluster_draws:
        Option<HashMap<u64, Vec<VirtualGeometryClusterRasterDraw>>>,
}
