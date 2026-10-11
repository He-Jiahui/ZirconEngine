use crate::core::math::{Real, UVec2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 平面相机的一帧输入；拖拽像素按 viewport_size 与缩放状态换算为世界位移，键盘平移另按秒积分。
///
/// # 字段与使用时机
///
/// `delta_seconds` 是键盘平移和旋转的积分秒数，负值按零处理；非零 `pan_axis` 使用相机右向和上向构成归一化方向。
/// `drag_delta` 是指针差量，按视口最短边、持续缩放状态和拖拽系数折算世界位移。
/// `rotate_axis` 与旋转速度相乘，`zoom_delta` 调整持续缩放状态；拖拽和缩放不乘帧秒数。
///
/// # 顺序与边界
///
/// 控制器依次应用键盘平移、拖拽、旋转和缩放，所以本次拖拽使用旋转和缩放更新之前的方向与状态。
/// 此输入没有焦点字段；宿主是否投递动作由调用方决定，控制器另按自身启用状态门控。
/// 输入值不修改相机，持续缩放不保存在这里。
///
/// # 参考
///
/// 参见 [`crate::input::camera_controller::PanCameraController::update`] 与
/// [`crate::core::framework::camera_controller::PanCameraState`]。
pub struct PanCameraInput {
    pub delta_seconds: Real,
    pub pan_axis: Vec2,
    pub drag_delta: Vec2,
    pub zoom_delta: Real,
    pub rotate_axis: Real,
    pub viewport_size: UVec2,
}

impl Default for PanCameraInput {
    /// 构造没有平面导航动作的帧输入基值。
    ///
    /// # 返回值与使用时机
    ///
    /// 返回零积分时间、零平移／拖拽／旋转／缩放输入和一像素尺寸，供调用方补充本帧动作。
    /// 尺寸未读取宿主视口；若投递拖拽，调用方应提供实际尺寸以得到所需位移换算。
    ///
    /// # 生命周期与参考
    ///
    /// 构造不清空控制器已有缩放状态，也不把输入变换的缩放同步到状态。
    /// 参见 [`PanCameraInput`] 与 [`crate::input::camera_controller::PanCameraController::update`]。
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            pan_axis: Vec2::ZERO,
            drag_delta: Vec2::ZERO,
            zoom_delta: 0.0,
            rotate_axis: 0.0,
            viewport_size: UVec2::ONE,
        }
    }
}
