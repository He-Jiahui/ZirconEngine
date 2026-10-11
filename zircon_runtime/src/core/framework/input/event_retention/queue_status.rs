//! 定义即时帧事件的保留数与合并数快照。
//!
//! # 使用时机与生命周期
//! 管理器查询帧缓冲时填充这些标量。查询后的提交、排空和帧窗口清理可能改变缓冲，
//! 此前返回的值保持不变；原始录制的容量和丢弃周期由独立记录器维护。
//!
//! # 参考
//! [`InputEventQueueStatus`]、[`crate::core::framework::input::InputManager::event_queue_status`]。
use serde::{Deserialize, Serialize};

/// 描述当前帧的即时事件队列；连续光标移动和鼠标位移可合并，合并计数在下一帧开始时归零。
///
/// 保存一次查询时即时帧缓冲的事件槽位数与本统计窗口的合并次数。
///
/// # 使用时机与统计范围
/// 默认输入管理器查询帧缓冲时返回本值。相邻 `CursorMoved` 保留最新位置，
/// 相邻 `MouseMotion` 累加位移；其它事件或不同指针事件形成合并屏障。
/// 帧缓冲使用 `Vec`，原始录制的条目容量不会限制它。
///
/// # 默认值、约束与生命周期
/// `Default` 得到两个零值；公开字段、复制和 Serde 只处理数值，不验证真实缓冲状态。
/// 排空使保留数归零，但合并数持续到下一次 `begin_frame` 才清零；因此两字段不能相加
/// 作为全程提交总数。返回值独立拥有标量，不携带集合借用、状态锁或订阅。
/// 接口的默认查询也返回零值；各管理器实现需说明自己支持的统计语义。
///
/// # 参考
/// [`crate::core::framework::input::InputManager::event_queue_status`]、
/// [`crate::core::framework::input::InputManager::drain_events`]、
/// [`crate::core::framework::input::InputManager::begin_frame`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventQueueStatus {
    /// 查询时仍在即时队列内、尚未排空的事件槽位数。
    ///
    /// # 约束与生命周期
    /// 默认帧缓冲将 `Vec` 长度转换为 `u32`；超出表示范围会截断。
    /// 已合并的相邻同类指针样本只占一个槽位。排空后再次查询为零，旧快照保持原值；
    /// 此字段不统计原始提交总数、容量丢弃或原始录制条目。
    ///
    /// # 参考
    /// [`crate::core::framework::input::InputManager::event_queue_status`]、
    /// [`crate::core::framework::input::InputManager::drain_events`]。
    pub retained_events: u32,
    /// 自构造或最近一次帧窗口清理以来，成功并入队列末项的样本次数。
    ///
    /// # 约束与生命周期
    /// 默认帧缓冲每次合并饱和加一，达到 `u64::MAX` 后保持该值。
    /// 排空事件不会重置计数，`begin_frame` 才归零；它是窗口累计量，不能当作每次排空后的新增量。
    /// 快照只复制计数，不保留计数器借用。
    ///
    /// # 参考
    /// [`crate::core::framework::input::InputManager::begin_frame`]、
    /// [`crate::core::framework::input::InputManager::event_queue_status`]。
    pub coalesced_events: u64,
}
