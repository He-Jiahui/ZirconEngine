use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationRuntimeStats {
    pub loaded_nav_meshes: usize,
    pub active_agents: usize,
    pub active_obstacles: usize,
    pub active_off_mesh_links: usize,
    pub active_off_mesh_bridges: usize,
}
