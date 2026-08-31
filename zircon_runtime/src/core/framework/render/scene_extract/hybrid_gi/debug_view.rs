use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderHybridGiDebugView {
    None,
    Cards,
    SurfaceCache,
    VoxelClipmap,
    InputSet,
}

impl Default for RenderHybridGiDebugView {
    fn default() -> Self {
        Self::None
    }
}
