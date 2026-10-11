use std::f32::consts::PI;

use crate::core::math::Real;

/// 键盘平移速度按世界单位/秒、旋转速度按弧度/秒解释；拖拽速度按视口最短边和当前缩放折算。
#[derive(Clone, Copy, Debug, PartialEq)]
///
/// # 参数含义与约束
///
/// `pan_speed` 是世界单位每秒，`rotation_speed` 是弧度每秒；键盘方向会归一化。
/// `drag_pan_speed` 配合持续缩放和视口最短边折算指针差量，不乘帧秒数。
/// `zoom_speed` 把缩放增量换成状态差量；它和旋转速度没有被强制取非负，符号由调用方选择。
/// 控制器使用时将平移和拖拽速度取非负、将 `min_zoom`／`max_zoom` 排序，但本值不验证有限性或正缩放。
/// 如调用方需要始终保持正缩放，应提供符合该要求的边界，不能把构造完成当成校验通过。
///
/// # 生命周期与参考
///
/// 设置按值保存，不自动同步输入变换和缩放状态；缩放动作会用一个标量覆盖输出变换的三个缩放分量。
/// 参见 [`crate::input::camera_controller::PanCameraController::settings_mut`] 与
/// [`crate::core::framework::camera_controller::PanCameraState`]。
pub struct PanCameraSettings {
    pub pan_speed: Real,
    pub zoom_speed: Real,
    pub min_zoom: Real,
    pub max_zoom: Real,
    pub rotation_speed: Real,
    pub drag_pan_speed: Real,
}

impl Default for PanCameraSettings {
    /// 提供新平面控制器使用的默认运动和缩放策略。
    ///
    /// # 返回值
    ///
    /// 返回每秒 500 世界单位平移、每秒 π 弧度旋转，拖拽系数为一、缩放系数为 0.1。
    /// 默认缩放区间为 0.1 到 5；这是返回的策略值，不是对公开字段后续修改的校验。
    ///
    /// # 使用时机与参考
    ///
    /// 结果按值保存到控制器，不改变已有实例，也不从输入相机提取持续缩放。
    /// 参见 [`PanCameraSettings`] 与 [`crate::input::camera_controller::PanCameraController::new`]。
    fn default() -> Self {
        Self {
            pan_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.1,
            max_zoom: 5.0,
            rotation_speed: PI,
            drag_pan_speed: 1.0,
        }
    }
}
