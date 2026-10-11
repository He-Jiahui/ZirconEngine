use crate::core::math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 轨道目标属于持续状态；视口平移必须同时更新相机位置和 target，后续旋转/缩放才围绕同一焦点。
///
/// # 状态与使用时机
///
/// `target` 是持续目标的世界坐标；平移同步移动它，聚焦动作替换它，后续旋转和缩放继续使用它。
/// `enabled` 为假时动作更新不修改目标或变换，但不会重置目标；默认状态启用且目标为原点。
///
/// # 生命周期边界
///
/// 状态可复制，不携带场景身份或代际，也不自动跟踪相机位置。
/// 调用方接入已有相机或跨场景复用时，须协调目标与变换的坐标空间。
///
/// # 参考
///
/// 参见 [`crate::input::camera_controller::OrbitCameraController::state_mut`] 与
/// [`crate::core::framework::camera_controller::OrbitCameraInput::focus`]。
pub struct OrbitCameraState {
    pub enabled: bool,
    pub target: Vec3,
}

impl Default for OrbitCameraState {
    /// 建立以世界原点为持续目标的启用轨道状态。
    ///
    /// # 返回值与约束
    ///
    /// 返回 `enabled = true` 和零目标点，不从已有相机或选择对象计算目标。
    /// 接入场景前，调用方须协调目标与相机的世界坐标空间；此方法不重定位相机。
    ///
    /// # 生命周期与参考
    ///
    /// 结果是独立状态值，不携带场景身份，也不清空其他控制器的目标。
    /// 参见 [`OrbitCameraState`] 与 [`crate::input::camera_controller::OrbitCameraController::state_mut`]。
    fn default() -> Self {
        Self {
            enabled: true,
            target: Vec3::ZERO,
        }
    }
}
