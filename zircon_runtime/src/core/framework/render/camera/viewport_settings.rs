use serde::{Deserialize, Serialize};

use super::{DisplayMode, ProjectionMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 编辑器视口的预览意图；场景提取据此选择投影与预览效果，不能当作持久相机配置。
pub struct ViewportRenderSettings {
    pub projection_mode: ProjectionMode,
    pub display_mode: DisplayMode,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
}

impl Default for ViewportRenderSettings {
    fn default() -> Self {
        Self {
            projection_mode: ProjectionMode::Perspective,
            display_mode: DisplayMode::Shaded,
            preview_lighting: true,
            preview_skybox: true,
        }
    }
}
