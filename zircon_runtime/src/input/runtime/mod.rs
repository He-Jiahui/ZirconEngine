//! 挂载运行时输入实现，并提供其类型重导出。
//!
//! # 模块责任
//! 共享输入协议由核心接口定义；这里组合默认服务实现、动作求值器、事件缓冲与录制工具。[`DefaultInputManager`] 与 [`DefaultInputActionManager`] 的服务声明由模块描述符提供，实际注册由 Core 调用者通过 [`crate::core::CoreRuntime::register_module`] 发起，仅导入这些类型不会注册或推进帧。
//!
//! # 可见性与所有权
//! [`InputState`] 只在 crate 内重导出，默认输入管理器拥有并同步访问其状态。[`InputDriver`] 当前只是注册表身份，宿主事件循环负责采样；动作管理器通过输入帧快照求值。
//!
//! # 录制与调用时机
//! [`InputRecording`]、[`InputRecordingFrame`]、[`InputReplayCursor`] 与 [`InputReplayFrameReport`] 是显式调用的录制及回放工具，不创建独立输入线程。
//!
//! # 参考
//! 参见 [`crate::input::module_descriptor`]、[`InputActionEvaluator`] 与 [`DefaultInputManager`]。
mod action_evaluator;
mod default_input_action_manager;
mod default_input_manager;
mod event_buffer;
mod input_driver;
mod input_state;
mod recording;

pub use action_evaluator::InputActionEvaluator;
pub use default_input_action_manager::DefaultInputActionManager;
pub use default_input_manager::DefaultInputManager;
pub use input_driver::InputDriver;
pub(crate) use input_state::InputState;
pub use recording::{
    InputRecording, InputRecordingFrame, InputReplayCursor, InputReplayFrameReport,
};
