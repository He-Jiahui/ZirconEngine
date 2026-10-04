//! 作者变换是场景文档与 World 层级之间的稳定表示；保存/加载都应保留相同的局部空间语义。

use crate::core::math::Real;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformAsset {
    pub translation: [Real; 3],
    pub rotation: [Real; 4],
    pub scale: [Real; 3],
}

impl Default for TransformAsset {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}
