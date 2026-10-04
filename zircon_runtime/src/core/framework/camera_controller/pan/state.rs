use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 平面控制器跨帧的缩放状态；缩放动作会同步写入输出变换的 scale。
pub struct PanCameraState {
    pub enabled: bool,
    pub zoom_factor: Real,
}

impl Default for PanCameraState {
    fn default() -> Self {
        Self {
            enabled: true,
            zoom_factor: 1.0,
        }
    }
}
