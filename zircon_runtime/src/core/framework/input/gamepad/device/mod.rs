//! 定义手柄编号与单次连接状态通知，使输入状态和设备输出请求使用同一套共享值。
//!
//! # 使用时机
//!
//! 宿主枚举或发现连接变化时创建连接通知；按钮、轴与震动请求使用相同编号配对目标。
//!
//! # 约束
//!
//! 模块导出协议数据。编号由事件生产者提供，本模块不分配全局设备身份，不持有驱动或校验实时连接。
//!
//! # 所有权与生命周期
//!
//! GamepadId 可按值复制；GamepadConnectionInfo 拥有可选名称字符串。保存这些值只保存观察或路由信息。
//!
//! # 参考
//!
//! [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadConnectionInfo`]、[`crate::core::framework::input::InputEvent`]、[`crate::core::framework::input::GamepadRumbleRequest`]
mod connection_info;
mod id;

pub use self::{connection_info::GamepadConnectionInfo, id::GamepadId};
