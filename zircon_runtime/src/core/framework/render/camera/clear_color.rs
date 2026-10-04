use crate::core::math::Vec4;
use serde::{Deserialize, Serialize};

/// 相机组件经 CameraRenderDescriptor 转为栈清屏策略：Default 使用天空盒，None 不清屏，Color 指定常量值。
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
