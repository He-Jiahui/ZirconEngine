use std::f32::consts::FRAC_PI_2;

use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 自由相机手感参数由控制器解释；速度按秒积分，俯仰边界使用弧度，滚轮倍率跨帧保留。
///
/// # 参数含义与使用约束
///
/// `walk_speed`、`run_speed` 是世界单位每秒的速度，`sensitivity` 把视角指针增量按其值除以 180 换算弧度。
/// `scroll_factor` 是滚轮速度倍率的指数系数；倍率会保存在状态中，随后的移动继续使用它。
/// `friction` 是无移动输入时指数衰减的每秒系数；`pitch_min`、`pitch_max` 是弧度边界。
/// 控制器使用时将速度和摩擦取非负、将俯仰边界排序；本值没有构造时校验，不能据此推断任意数值都合法。
///
/// # 生命周期
///
/// 设置按值保存进控制器；后来修改原副本不改变已有控制器。
/// 调整已有实例通过其设置入口完成，不会自动重建姿态、速度或滚轮倍率。
///
/// # 参考
///
/// 参见 [`crate::input::camera_controller::FreeCameraController::settings_mut`] 与
/// [`crate::core::framework::camera_controller::FreeCameraState`]。
pub struct FreeCameraSettings {
    pub sensitivity: Real,
    pub walk_speed: Real,
    pub run_speed: Real,
    pub scroll_factor: Real,
    pub friction: Real,
    pub pitch_min: Real,
    pub pitch_max: Real,
}

impl Default for FreeCameraSettings {
    /// 提供新自由相机控制器使用的默认运动与视角策略。
    ///
    /// # 返回值
    ///
    /// 返回走速 5、跑速 15 的世界单位每秒配置，视角系数为 0.2，俯仰边界为正负 π/2。
    /// 滚轮指数系数为 0.04879016，无移动输入时每秒摩擦系数为 40；各字段的解释由控制器承担。
    ///
    /// # 使用时机与参考
    ///
    /// 结果是独立设置值，不改变已有控制器或验证后续修改；构造控制器时按值保存。
    /// 参见 [`FreeCameraSettings`] 与 [`crate::input::camera_controller::FreeCameraController::new`]。
    fn default() -> Self {
        Self {
            sensitivity: 0.2,
            walk_speed: 5.0,
            run_speed: 15.0,
            scroll_factor: 0.04879016,
            friction: 40.0,
            pitch_min: -FRAC_PI_2,
            pitch_max: FRAC_PI_2,
        }
    }
}
