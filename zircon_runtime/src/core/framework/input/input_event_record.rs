//! 定义原始输入记录的周期序号、诊断时间和拥有型事件载荷。
//!
//! # 职责与使用时机
//! InputManager 的记录排空接口返回这些值；DefaultInputManager 使用的有界记录器在启用且保留策略允许时克隆提交事件，录制捕获与回放消费所得记录。公开构造也允许其它生产者提供标签。
//!
//! # 边界与生命周期
//! 本模块不持有记录队列，不实施容量策略，也不验证时间或全局身份。记录独立拥有事件，生命周期与管理器、即时合并缓冲及录制启用周期分别解释；序号和时间不构成回放调度协议。
//!
//! # 参考
//! 参见 [InputEventRecord](super::InputEventRecord)、[InputManager::drain_event_records_with_status()](super::InputManager::drain_event_records_with_status)。
use serde::{Deserialize, Serialize};

use super::InputEvent;

/// 保存一次被原始记录器保留的提交事件及诊断标签。
///
/// # 使用时机与职责
/// DefaultInputManager 使用的原始记录器启用且有容量时在即时合并前克隆提交事件；录制捕获取走这些记录，回放按容器中的事件顺序重新提交。
///
/// # 约束
/// 字段公开且派生 Deserialize 不验证唯一性、时间顺序、事件归一化或录制完整性。记录器序号在当前启用周期单调不减并可饱和重复，时间来自系统墙钟可能回退；合成回放记录可用零时间。两个标签都不是跨管理器/跨周期全局身份或回放计时协议。
///
/// # 生命周期
/// 记录按值拥有事件，存活不依赖管理器锁或源队列；快照、即时队列和原始记录具有独立保留/消费边界。
///
/// # 参考
/// 参见 [InputEvent](super::InputEvent)、[InputEventRecordingStatus](super::InputEventRecordingStatus)、[InputManager::drain_event_records()](super::InputManager::drain_event_records)、[InputManager::drain_event_records_with_status()](super::InputManager::drain_event_records_with_status)。
/// 可选录制中的原始事件与采集顺序；帧队列合并不会改写记录，回放按 `event` 重新提交。
/// 序号只在一次启用周期内单调，时间戳用于诊断而非回放调度。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputEventRecord {
    /// 记录器当前启用周期内的提交序号标签。
    ///
    /// # 使用时机与职责
    /// 由记录生产者填写；消费方按自身录制周期和容器上下文解释。
    ///
    /// # 约束
    /// DefaultInputManager 使用的原始记录器首次为 1，以饱和加一递增；达到 u64::MAX 后可重复。禁用或关闭后再启用重置周期，已启用调容量不重置；零容量提交仍消耗序号并计丢弃。公开构造或合成回放标签可不遵从这些规则，不保证全局唯一或跨周期可比较。
    ///
    /// # 生命周期
    /// 按值归本记录拥有；不借用管理器、录制队列或宿主输入缓冲。
    ///
    /// # 参考
    /// 参见 [InputEventRecordingConfig](super::InputEventRecordingConfig)、[InputEventRecordingStatus](super::InputEventRecordingStatus)。
    pub sequence: u64,
    /// 记录生成时的毫秒诊断标签。
    ///
    /// # 使用时机与职责
    /// 由记录生产者填写；消费方按自身录制周期和容器上下文解释。
    ///
    /// # 约束
    /// DefaultInputManager 使用的原始记录器从 SystemTime 与 UNIX_EPOCH 的差取毫秒，时钟差失败取零，整数转换未提供饱和保证。系统时钟可回退；此字段不承诺严格/非严格时间递增，也不用于回放等待或排序。合成回放记录填写零。
    ///
    /// # 生命周期
    /// 按值归本记录拥有；不借用管理器、录制队列或宿主输入缓冲。
    ///
    /// # 参考
    /// 参见 [InputEvent](super::InputEvent)、[InputEventRecord](super::InputEventRecord)。
    pub timestamp_millis: u64,
    /// 被记录器保留的原始拥有型提交事件。
    ///
    /// # 使用时机与职责
    /// 由记录生产者填写；消费方按自身录制周期和容器上下文解释。
    ///
    /// # 约束
    /// DefaultInputManager 使用的原始记录器克隆提交载荷，不用即时队列合并产物或处理后的快照值替换；关闭录制或容量丢弃时可能无记录。回放克隆此事件再提交，公开构造和 serde 仍不添加事件验证。
    ///
    /// # 生命周期
    /// 按值归本记录拥有；不借用管理器、录制队列或宿主输入缓冲。
    ///
    /// # 参考
    /// 参见 [InputEvent](super::InputEvent)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    pub event: InputEvent,
}
