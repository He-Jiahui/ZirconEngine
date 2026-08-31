use crate::core::math::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RenderCameraClearColor {
    Default,
    None,
    Color(Vec4),
}

impl Default for RenderCameraClearColor {
    fn default() -> Self {
        Self::Default
    }
}
