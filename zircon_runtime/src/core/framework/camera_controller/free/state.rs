use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 控制器跨帧保留的姿态与速度状态；重建控制器会丢失滚轮速度倍率和摩擦衰减过程。
///
/// # 状态与使用时机
///
/// `pitch`、`yaw` 是持续的弧度角，不会自动从传入变换同步；首次视角动作按这些值生成旋转。
/// `velocity` 存储沿相机右向、世界竖直方向和相机前向的速度分量，不是固定世界坐标速度。
/// `speed_multiplier` 跨帧保留滚轮倍率；默认倍率为一、速度和角度为零、控制器启用。
///
/// # 生命周期边界
///
/// 禁用只让更新保留输入变换，不清空已有角度、倍率和速度；重新启用会继续使用保存状态。
/// 此值可以复制，不持有相机或场景身份；接入已有相机、重置或跨场景复用时由调用方协调初始状态。
///
/// # 参考
///
/// 参见 [`crate::input::camera_controller::FreeCameraController::state_mut`] 与
/// [`crate::core::framework::camera_controller::FreeCameraInput`]。
pub struct FreeCameraState {
    pub enabled: bool,
    pub pitch: Real,
    pub yaw: Real,
    pub speed_multiplier: Real,
    pub velocity: Vec3,
}

impl Default for FreeCameraState {
    /// 建立新自由相机控制器的启用、静止初始状态。
    ///
    /// # 返回值与约束
    ///
    /// 返回零俯仰／偏航、零速度、速度倍率为一的状态；这些值不从现有相机变换提取。
    /// 接入已有姿态时由调用方协调角度，否则首次视角动作将按初始角度生成旋转。
    ///
    /// # 生命周期与参考
    ///
    /// 此方法只构造状态值，不重置任何既有实例，也不取得相机或窗口资源。
    /// 参见 [`FreeCameraState`] 与 [`crate::input::camera_controller::FreeCameraController::state_mut`]。
    fn default() -> Self {
        Self {
            enabled: true,
            pitch: 0.0,
            yaw: 0.0,
            speed_multiplier: 1.0,
            velocity: Vec3::ZERO,
        }
    }
}
