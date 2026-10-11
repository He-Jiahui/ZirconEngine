//! 定义运行时传给宿主的双电机强度值及 Add、Stop 震动请求。
//!
//! # 使用时机
//!
//! 调用者构造请求并通过 InputEvent 提交；有请求支持的输入实现保留它，宿主通过排空接口消费后再尝试驱动设备。
//!
//! # 约束
//!
//! new 与 add 保存原值；clamped 提供强度归一化。请求是输出意图，trait 的默认空排空适配没有提供硬件能力确认；实际设备查找、限额和平台失败由消费方承担。
//!
//! # 所有权与生命周期
//!
//! 强度及请求都是拥有型 Copy 值，无设备引用或运行中效果句柄。快照中的副本是观察视图，交付应使用管理器排空接口。
//!
//! # 参考
//!
//! [`crate::core::framework::input::GamepadRumbleIntensity`]、[`crate::core::framework::input::GamepadRumbleRequest`]、[`crate::core::framework::input::InputEvent::GamepadRumbleRequest`]、[`crate::core::framework::input::InputManager::drain_gamepad_rumble_requests()`]
mod intensity;
mod request;

pub use self::{intensity::GamepadRumbleIntensity, request::GamepadRumbleRequest};
