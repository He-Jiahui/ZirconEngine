use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshAssetUsage {
    pub main_world: bool,
    pub render_world: bool,
}

impl Default for MeshAssetUsage {
    fn default() -> Self {
        Self {
            main_world: true,
            render_world: true,
        }
    }
}
