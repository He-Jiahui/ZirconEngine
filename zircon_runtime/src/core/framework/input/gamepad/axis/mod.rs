//! 汇集模拟轴身份、设备轴消费键、整形设置和帧观察值，提供输入协议的公共轴入口。
//!
//! # 职责与入口
//!
//! GamepadAxis 与设备标签共同定位控件；GamepadAxisSettings 在事件进入缓存前处理原始采样。GamepadAxisState 表示读取时的已接受当前值，GamepadAxisTransition 提供当前帧的变化端点。
//! 私有子模块和再导出只组织协议名称。宿主轮询、连接维护、状态锁和动作求值由实际输入管理器及消费者推进。
//!
//! # 使用时机、约束与生命周期
//!
//! 设备事件解释、动作绑定和读取帧状态时使用这些拥有式值；它们不注册服务、不拥有设备，也不自行更新。
//! 当前默认管理器使用默认设置，跨 begin_frame 保留轴值并清理变化；公开 DTO 或其它管理器实现不自动获得这一生产策略。自定义配置和公开字段没有构造验证。
//!
//! # 参考
//!
//! 参见 [`GamepadAxis`]、[`GamepadAxisInput`]、[`GamepadAxisSettings`]、[`GamepadAxisState`]、[`GamepadAxisTransition`]、[`crate::core::framework::input::InputManager::frame_snapshot()`]。
#[path = "axis.rs"]
mod axis;
#[path = "input.rs"]
mod input;
#[path = "settings.rs"]
mod settings;
#[path = "state.rs"]
mod state;
#[path = "transition.rs"]
mod transition;

pub use self::{
    axis::GamepadAxis,
    input::GamepadAxisInput,
    settings::{
        GamepadAxisSettings, GAMEPAD_AXIS_CHANGE_THRESHOLD, GAMEPAD_AXIS_DEADZONE_LOWER,
        GAMEPAD_AXIS_DEADZONE_UPPER, GAMEPAD_AXIS_LIVEZONE_LOWER, GAMEPAD_AXIS_LIVEZONE_UPPER,
    },
    state::GamepadAxisState,
    transition::GamepadAxisTransition,
};
