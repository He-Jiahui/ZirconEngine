//! 定义原始输入录制配置、保留数和周期丢弃数的按值观察类型。
//!
//! # 使用时机与生命周期
//! 管理器查询或组合排空接口提供本次状态。后续记录、排空或配置更新可能改变记录器，
//! 旧值保持不变；状态不持有记录借用或锁。捕获辅助函数使用开关与累计丢弃数判断自身完整性，
//! 本类型不执行录制，也不验证全部输入历史。
//!
//! # 参考
//! [`InputEventRecordingStatus`]、[`crate::core::framework::input::InputManager::event_recording_status`]、
//! [`crate::core::framework::input::InputManager::drain_event_records_with_status`]。
use serde::{Deserialize, Serialize};

/// 与记录排空一起读取时用于判断回放是否完整；丢弃数累计到重新配置为关闭或重新启用。
///
/// 保存一次查询时的原始录制配置，以及当前录制周期的保留数和累计容量丢弃数。
///
/// # 默认值与使用时机
/// `Default` 得到 `enabled = false` 和三个零值；默认录制配置携带的容量另为 8,192。
/// 默认记录器从当前配置和 FIFO 填充状态。`Clone`/`Copy` 与 Serde 只处理标量，
/// 公开构造或反序列化不验证字段是否来自真实记录器。
///
/// # 生命周期与观察时点
/// 排空使保留数归零，累计丢弃数继续存在；关闭或从关闭切换到开启重置周期，
/// 持续开启的配置更新与帧窗口清理不会结束周期。
/// 默认输入管理器在同一个状态锁内先排空再查询，组合入口返回的 `retained_records` 因此为零；
/// 本次取得的记录数量应从同时返回的向量读取。其它实现需遵守接口合同并说明观察时点。
///
/// # 完整性边界
/// 开启 `native-host` 时，`crate::input::InputRecordingFrame::capture_from_manager`
/// 保存状态中的开关和累计丢弃数，`is_complete` 以开启且丢弃为零判断本次捕获。
/// 这项判断不证明全部输入来源或任意手工构造记录的历史完整。状态独立拥有数值，
/// 不携带记录借用、实时订阅或自身的线程同步能力；接口默认独立查询仅返回零值。
///
/// # 参考
/// [`crate::core::framework::input::InputEventRecordingConfig`]、
/// [`crate::core::framework::input::InputManager::event_recording_status`]、
/// [`crate::core::framework::input::InputManager::drain_event_records_with_status`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventRecordingStatus {
    /// 查询时录制器配置中的开启标志。
    ///
    /// # 约束与生命周期
    /// 默认记录器复制当前开关。`true` 且零容量时仍可没有保留记录，`false` 不关闭即时帧队列。
    /// 捕获完整性还需累计丢弃为零；本字段只观测数值，不控制或借用记录器。
    /// 默认零状态本身不能证明管理器是否支持录制。
    ///
    /// # 参考
    /// [`crate::core::framework::input::InputEventRecordingConfig`]、
    /// [`crate::core::framework::input::InputManager::event_recording_status`]。
    pub enabled: bool,
    /// 查询时保存的原始记录条目容量，包括关闭配置携带的容量。
    ///
    /// # 约束与生命周期
    /// 默认记录器原样复制当前 `u32` 容量，允许零；它不表示已保留数量或预分配内存。
    /// 后续配置更新不会改变旧快照。状态默认值的零容量与默认配置的 8,192 不同，
    /// 公开字段值也不能证明某实现已经完成裁剪。
    ///
    /// # 参考
    /// [`crate::core::framework::input::InputEventRecordingConfig`]、
    /// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
    pub capacity: u32,
    /// 查询时仍在原始录制 FIFO 内等待排空的记录条目数。
    ///
    /// # 约束与观察时点
    /// 默认记录器将 FIFO 长度转换为 `u32`；正常配置和记录路径以 `u32` 容量限制长度。
    /// 此值不含已排空或丢弃的记录，也不计帧内合并。默认管理器的组合入口先排空再取状态，
    /// 因此返回零；本次取得的数量应从返回向量读取。旧快照不随后续记录更新，也不借用 FIFO。
    ///
    /// # 参考
    /// [`crate::core::framework::input::InputManager::drain_event_records_with_status`]、
    /// [`crate::core::framework::input::InputManager::event_recording_status`]。
    pub retained_records: u32,
    /// 当前录制周期因容量而丢弃的原始记录累计数，使用 `u64` 饱和计数。
    ///
    /// # 更新与生命周期
    /// 默认记录器在零容量提交、满载淘汰和持续开启缩容时增加计数，达到 `u64::MAX` 后保持该值。
    /// 排空和帧边界不清零；关闭或从关闭切换到开启重置周期，持续开启的增容或重复配置保留计数。
    /// 不同帧保存的是累计观察，不能相加作为总损失。条件启用的 `crate::input::InputRecording`
    /// 取最大观察值；它也不将多个重置周期或多个记录器的损失相加。
    /// 零值需结合真实开启状态与同次捕获来源判断；状态不借用计数器，周期重置后的值不能直接
    /// 视为旧周期的延续。
    ///
    /// # 参考
    /// [`crate::core::framework::input::InputManager::event_recording_status`]、
    /// [`crate::core::framework::input::InputManager::drain_event_records_with_status`]。
    pub discarded_records: u64,
}
