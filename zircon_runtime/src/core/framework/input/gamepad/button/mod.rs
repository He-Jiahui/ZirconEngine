//! 统一手柄按钮的控件身份、模拟幅值整形、滞回判定参数与当前幅值视图，供输入事件、帧快照和动作绑定使用。
//!
//! # 入口与职责
//!
//! [`GamepadButton`] 表示设备内的按钮标签，实际绑定须与设备编号组合；[`GamepadButtonAxisSettings`] 接收原始模拟值并决定是否接受一次幅值更新。
//! [`GamepadButtonSettings`] 根据调用方提供的当前按住状态判断候选转换；[`GamepadButtonValueState`] 保存已接受幅值的拥有式观察值。
//! 默认输入管理器先做幅值过滤，再更新按住与边沿集合；动作求值读取这些布尔集合，当前实现不从幅值视图直接取得按钮动作强度。UI 导航还可使用独立的宿主事件路由。
//!
//! # 使用时机与生命周期
//!
//! 在设备事件解释、配置值构造和读取帧状态时使用这些协议值；它们不持有设备或管理器借用，也不自行轮询、注册或派发事件。
//! 配置构造与派生反序列化不执行字段验证；公开方法的输入准入和历史比较条件须分别遵守。幅值视图代表读取时已有状态，持有副本不会随后续事件自动更新。
//! 子模块与再导出只组织名称；设备连接、帧边沿清理和服务生命周期由实际输入管理器及宿主推进。
//!
//! # 参考
//!
//! 参见 [`GamepadButton`]、[`GamepadButtonSettings`]、[`GamepadButtonAxisSettings`]、[`GamepadButtonValueState`] 与 [`crate::core::framework::input::InputManager::frame_snapshot()`]。
mod axis_settings;
mod button;
mod settings;
mod value_state;

pub use self::{
    axis_settings::{
        GamepadButtonAxisSettings, GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD, GAMEPAD_BUTTON_AXIS_HIGH,
        GAMEPAD_BUTTON_AXIS_LOW,
    },
    button::GamepadButton,
    settings::{
        GamepadButtonSettings, GAMEPAD_BUTTON_PRESS_THRESHOLD, GAMEPAD_BUTTON_RELEASE_THRESHOLD,
    },
    value_state::GamepadButtonValueState,
};
