use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 平面控制器跨帧的缩放状态；缩放动作会同步写入输出变换的 scale。
///
/// # 状态与使用时机
///
/// `zoom_factor` 是拖拽和缩放操作共用的持续标量，默认值为一；它不会自动从输入变换的 `scale` 同步。
/// 非零缩放动作更新这个值并将输出缩放写成三个相同分量；零缩放输入不触发该同步。
/// `enabled` 为假时保留输入变换和原状态，不执行重置。
///
/// # 生命周期与参考
///
/// 状态可复制，不持有场景或相机；接入已有缩放、重置或跨场景复用时由调用方协调初值。
/// 参见 [`crate::input::camera_controller::PanCameraController::state_mut`] 与
/// [`crate::core::framework::camera_controller::PanCameraInput`]。
pub struct PanCameraState {
    pub enabled: bool,
    pub zoom_factor: Real,
}

impl Default for PanCameraState {
    /// 建立启用且持续缩放为一的平面控制器状态。
    ///
    /// # 返回值与约束
    ///
    /// 返回 `enabled = true` 与 `zoom_factor = 1`，不从输入相机的三个缩放分量计算标量。
    /// 接入已有缩放时由调用方协调初值；非零缩放动作才会把状态写入输出变换。
    ///
    /// # 生命周期与参考
    ///
    /// 此方法只构造独立状态值，不修改场景或重置其他实例。
    /// 参见 [`PanCameraState`] 与 [`crate::input::camera_controller::PanCameraController::state_mut`]。
    fn default() -> Self {
        Self {
            enabled: true,
            zoom_factor: 1.0,
        }
    }
}
