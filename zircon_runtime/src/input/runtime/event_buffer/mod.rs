//! 把帧内瞬时事件缓冲与可选原始输入记录器接入运行时输入状态。
//!
//! # 责任与入口
//! [`super::input_state::InputState`] 拥有 [`FrameEventBuffer`] 与 [`InputEventRecorder`]。
//! [`crate::input::DefaultInputManager`] 在状态锁内先更新输入状态、尝试记录原始事件，再提交到帧缓冲。
//! 本模块声明两个私有子模块，并仅向父运行时作用域重导出类型，不创建或执行记录器。
//!
//! # 保留规则与生命周期
//! 帧缓冲可合并相邻同类指针事件；排空转移事件，帧边界清空未消费事件并重置合并计数。
//! 原始记录器默认关闭，启用后按独立容量保留未合并事件；帧边界与帧缓冲排空均不清空它。
//! 原始记录排空保留配置、序号和丢弃计数；关闭或从关闭切换为启用时重置记录周期，持续启用时缩容可淘汰旧记录。
//!
//! # 参考
//! 参见 [`super::input_state::InputState`]、[`crate::input::DefaultInputManager`] 与 [`crate::input::InputRecordingFrame::capture_from_manager`]。
/// 接入负责帧内事件保留、相邻指针样本合并及帧窗口重置的私有实现。
///
/// # 使用时机与生命周期
/// 运行时状态持有其缓冲；提交、排空和帧边界由管理器转发，帧窗口不管理原始记录周期。
///
/// # 参考
/// 参见 [`crate::input::runtime::event_buffer::FrameEventBuffer`] 与 [`crate::input::DefaultInputManager`]。
mod frame;
/// 接入按可选配置保留原始事件、维护容量丢弃计数的私有实现。
///
/// # 使用时机与生命周期
/// 管理器在瞬时帧事件合并前尝试录制；原始记录独立跨帧保留，排空或配置更新按记录器规则处理。
///
/// # 参考
/// 参见 [`crate::input::runtime::event_buffer::InputEventRecorder`] 与 [`crate::input::InputRecordingFrame::capture_from_manager`]。
mod recorder;

pub(super) use frame::FrameEventBuffer;
pub(super) use recorder::InputEventRecorder;
