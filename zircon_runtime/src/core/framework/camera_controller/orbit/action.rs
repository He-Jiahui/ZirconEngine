use crate::core::math::{Real, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 视口导航传给轨道控制器的互斥动作；Focus 更新目标但不必立即改变相机变换。
///
/// # 动作与状态边界
///
/// 一次更新只消费一种动作。`Orbit` 与 `Pan` 的两点须采用同一指针坐标空间，控制器使用当前减前次的差量。
/// `Pan` 同时平移相机和持续目标；`Zoom` 沿相机当前前向移动，默认控制器仅消费增量的符号。
/// `Focus` 替换持续目标，不立即重新定位或旋转相机；内部目标变化不能由输出的 `changed` 标志证明。
///
/// # 参考
///
/// 参见 [`crate::core::framework::camera_controller::OrbitCameraInput`] 与
/// [`crate::input::camera_controller::OrbitCameraController::update`]。
pub enum OrbitCameraAction {
    None,
    Orbit { previous: Vec2, current: Vec2 },
    Pan { previous: Vec2, current: Vec2 },
    Zoom { delta: Real },
    Focus { target: Vec3 },
}

impl Default for OrbitCameraAction {
    /// 提供不请求轨道导航动作的默认值。
    ///
    /// # 返回值与边界
    ///
    /// 返回 `None` 动作；构造或随后消费这个动作不会把控制器持续目标重置为原点。
    /// 它不读取输入、执行相机更新或改变宿主焦点。
    ///
    /// # 参考
    ///
    /// 参见 [`OrbitCameraAction`] 与 [`crate::core::framework::camera_controller::OrbitCameraInput::none`]。
    fn default() -> Self {
        Self::None
    }
}
