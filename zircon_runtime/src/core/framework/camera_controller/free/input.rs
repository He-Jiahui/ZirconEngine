use crate::core::math::{Real, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 宿主为一帧自由相机操作汇总的输入；焦点限制视角和光标抓取，移动速度仍由控制器状态连续积分。
///
/// # 使用时机与字段约束
///
/// 每次自由相机更新提交一份输入值；它不保存跨帧速度，也不读取宿主状态。
/// `delta_seconds` 是本次积分的秒数，控制器将负值按零处理；`movement_axis` 是方向权重，
/// 非零向量会归一化，长度不直接放大速度。其分量分别沿相机右向、世界竖直方向和相机前向推进。
/// `look_delta` 是指针增量，按 `sensitivity / 180` 换算弧度；它仅在焦点有效且视角操作或抓取激活时使用。
/// `scroll_lines` 调整持续速度倍率，`run` 选择奔跑速度。
///
/// # 焦点与生命周期边界
///
/// `focus_active` 限制视角更新，不自动停止移动或摩擦积分；`cursor_grab_changed` 才触发抓取意图输出。
/// 输入值中的抓取标志是调用方报告的状态，不证明宿主已经抓取光标。
/// 默认输入焦点有效，其余动作量为零、动作开关为假；持续状态由控制器保留。
///
/// # 参考
///
/// 参见 [`crate::input::camera_controller::FreeCameraController::update`] 与
/// [`crate::core::framework::camera_controller::FreeCameraState`]。
pub struct FreeCameraInput {
    pub delta_seconds: Real,
    pub movement_axis: Vec3,
    pub look_delta: Vec2,
    pub scroll_lines: Real,
    pub run: bool,
    pub look_active: bool,
    pub focus_active: bool,
    pub cursor_grab_active: bool,
    pub cursor_grab_changed: bool,
}

impl Default for FreeCameraInput {
    /// 构造没有新动作、输入焦点有效的自由相机输入。
    ///
    /// # 返回值与使用时机
    ///
    /// 返回零积分时间、零移动／视角／滚轮输入，运行、视角和抓取相关开关为假。
    /// 调用方用它作为本帧输入的基值，再按真实焦点和动作补充字段；默认有效焦点不是宿主焦点回执。
    ///
    /// # 生命周期与参考
    ///
    /// 返回值不清空控制器已有速度或角度；无新移动输入时，持续速度仍由更新路径衰减。
    /// 参见 [`crate::input::camera_controller::FreeCameraController::update`] 与 [`FreeCameraInput`]。
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            movement_axis: Vec3::ZERO,
            look_delta: Vec2::ZERO,
            scroll_lines: 0.0,
            run: false,
            look_active: false,
            focus_active: true,
            cursor_grab_active: false,
            cursor_grab_changed: false,
        }
    }
}
