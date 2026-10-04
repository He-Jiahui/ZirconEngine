//! 网格用途是作者对静态/动态上传策略的提示；资源流据此选取使用方式，但几何有效性仍由 MeshAsset 校验。

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
