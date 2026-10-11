//! 公开输入帧队列的计数快照，以及可选原始录制的配置和状态。
//!
//! # 职责与使用时机
//! `queue_status` 描述即时队列；`recording_config` 提供录制开关、条目容量和构造器；
//! `recording_status` 描述当前录制周期。这些值由 [`crate::core::framework::input::InputManager`]
//! 的配置、查询和记录消费接口使用，模块声明与重导出只提供统一类型路径。
//!
//! # 生命周期与边界
//! 运行时管理器分别拥有即时帧缓冲和原始记录器，并在可能合并瞬时事件前尝试保存原始记录。
//! 配置需要交给管理器应用；状态按值保存查询结果，后续提交、排空或帧清理不会更新旧快照。
//! 帧队列的统计窗口与录制周期独立。本模块公开数据契约，不持有缓冲或注册服务。
//!
//! # 参考
//! [`InputEventQueueStatus`]、[`InputEventRecordingConfig`]、[`InputEventRecordingStatus`]、
//! [`DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`]。
mod queue_status;
mod recording_config;
mod recording_status;

pub use queue_status::InputEventQueueStatus;
pub use recording_config::{InputEventRecordingConfig, DEFAULT_INPUT_EVENT_RECORDING_CAPACITY};
pub use recording_status::InputEventRecordingStatus;
