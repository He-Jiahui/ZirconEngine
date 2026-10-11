//! 暂存运行时用于帧内消费的瞬时 [`InputEvent`]，并合并相邻的同类指针样本。
//!
//! # 调用关系
//! [`crate::input::DefaultInputManager`] 在帧边界调用 [`FrameEventBuffer::begin_frame`]；提交事件时，先更新输入状态并尝试保存原始记录，再把事件移入 [`FrameEventBuffer::push`]。消费者通过 [`FrameEventBuffer::drain`] 取得当前保留的瞬时事件。
//!
//! # 状态与生命周期
//! 默认构造得到空缓冲，可以直接接收事件。相邻的绝对光标位置保留最新坐标，相邻的鼠标相对位移累加；不同事件形成合并屏障。排空转移事件所有权但保留合并计数，下次帧边界清空尚未排空的事件并重置计数。
//!
//! 管理器的原始记录器独立保留未合并的输入；清空或排空本缓冲不会排空原始记录。
//!
//! # 参考
//! [`FrameEventBuffer`]、[`InputEventQueueStatus`]
use crate::input::{InputEvent, InputEventQueueStatus};

// 帧内瞬时事件供宿主按顺序提取；仅合并相邻指针样本，按钮、触摸和失焦保持排序屏障。
/// 保存运行时当前统计窗口内尚未消费的瞬时事件及其合并次数。
///
/// # 使用时机
/// 默认构造时事件为空、合并计数为零，可以直接调用 [`Self::push`]。[`Self::begin_frame`] 用于清空当前窗口，并非首次提交事件的初始化前提。运行时输入状态持有本缓冲；[`crate::input::DefaultInputManager`] 在自己的状态锁内访问它。
///
/// # 状态与生命周期
/// 事件按提交顺序进入缓冲，相邻的同类指针样本可能被覆盖或累加。[`Self::drain`] 转移尚未消费的事件，不清零合并计数；[`Self::begin_frame`] 同时丢弃尚未消费的事件并重置计数。可变操作借用本缓冲，不要求调用者另行建立记录周期。
///
/// # 参考
/// [`InputEvent`]、[`InputEventQueueStatus`]、[`Self::status`]
#[derive(Debug, Default)]
pub(in crate::input::runtime) struct FrameEventBuffer {
    /// 当前帧尚未排空的事件；元素按提交顺序保存，连续可合并的指针样本占用一个槽位。
    events: Vec<InputEvent>,
    /// 当前帧成功合并的样本次数；计数使用饱和加法，并随 [`Self::begin_frame`] 清零。
    coalesced_events: u64,
}

impl FrameEventBuffer {
    /// 清空当前事件缓冲并把合并计数归零，为随后提交的事件建立新的统计窗口。
    ///
    /// # 调用时机与生命周期
    /// 由 [`crate::input::DefaultInputManager`] 的帧边界操作转发。每次调用都会丢弃当前尚未排空的事件，包括重复调用时已经提交到本窗口的事件；默认构造的缓冲可以在首次调用之前接收事件。
    ///
    /// # 返回
    /// 返回 `()`，表示本缓冲已经清空、合并计数已经重置。本方法不修改其它输入状态，也不排空独立的原始记录器。
    ///
    /// # 参考
    /// [`Self::push`]、[`Self::drain`]、[`Self::status`]
    pub(in crate::input::runtime) fn begin_frame(&mut self) {
        self.events.clear();
        self.coalesced_events = 0;
    }

    /// 消费一个瞬时事件，按提交顺序追加或合并到当前缓冲。
    ///
    /// # 参数
    /// `event` 按值传入；不能合并时由缓冲接管事件，相邻的 `CursorMoved` 覆盖末项绝对坐标，相邻的 `MouseMotion` 将相对位移累加到末项。
    ///
    /// # 调用时机与约束
    /// 默认构造后即可调用。[`crate::input::DefaultInputManager`] 在更新输入状态并尝试保存该事件的原始记录后调用本方法；本缓冲只维护瞬时事件。仅相邻且同属上述一种指针事件的两项能够合并，按钮、触摸、失焦及不同类型的事件会阻断合并。
    ///
    /// # 返回与状态
    /// 返回 `()`，表示事件已经追加，或其样本已合并到原末项；没有单独的错误返回值。每次合并令合并计数饱和增加一次。排空后缓冲没有末项，后续提交从新槽位开始；计数持续到 [`Self::begin_frame`] 才重置。
    ///
    /// # 参考
    /// [`Self::drain`]、[`Self::status`]、[`InputEvent`]
    pub(in crate::input::runtime) fn push(&mut self, event: InputEvent) {
        if let Some(previous) = self.events.last_mut() {
            match (previous, &event) {
                (
                    InputEvent::CursorMoved { x, y },
                    InputEvent::CursorMoved {
                        x: next_x,
                        y: next_y,
                    },
                ) => {
                    *x = *next_x;
                    *y = *next_y;
                    self.coalesced_events = self.coalesced_events.saturating_add(1);
                    return;
                }
                (
                    InputEvent::MouseMotion { delta_x, delta_y },
                    InputEvent::MouseMotion {
                        delta_x: next_x,
                        delta_y: next_y,
                    },
                ) => {
                    *delta_x += *next_x;
                    *delta_y += *next_y;
                    self.coalesced_events = self.coalesced_events.saturating_add(1);
                    return;
                }
                _ => {}
            }
        }
        self.events.push(event);
    }

    /// 排空当前保留的瞬时事件，并把事件所有权交给调用者。
    ///
    /// # 返回
    /// 返回按缓冲顺序排列的事件向量，其中相邻同类指针样本可能已经合并；它是当前保留的瞬时流。空向量表示调用时没有保留事件。缓冲随即为空，合并计数保持原值；返回向量独立拥有事件，后续提交或帧边界不会修改它。
    ///
    /// # 调用时机与生命周期
    /// 由 [`crate::input::DefaultInputManager`] 的 `drain_events` 转发，在消费当前瞬时流时调用。没有后续提交时再次调用返回空向量；后续提交可以继续进入缓冲。未排空的事件会被下一次 [`Self::begin_frame`] 丢弃，独立的原始记录器不受本操作影响。
    ///
    /// # 参考
    /// [`Self::push`]、[`Self::status`]、[`InputEvent`]
    pub(in crate::input::runtime) fn drain(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.events)
    }

    /// 返回当前缓冲的计数快照，供调用者观察保留事件和合并操作。
    ///
    /// # 返回与时间边界
    /// [`InputEventQueueStatus`] 的 `retained_events` 是调用时的队列长度转换为 `u32` 的结果；长度超出 `u32` 可表示范围时会截断，不能作为任意规模队列的精确长度保证。`coalesced_events` 是构造以来或最近一次 [`Self::begin_frame`] 以来的合并次数，使用饱和计数，排空不会重置它。
    ///
    /// 本方法不修改或排空缓冲。返回值只描述调用瞬间，后续提交、排空或帧边界可能改变状态。
    ///
    /// # 参考
    /// [`Self::drain`]、[`Self::push`]
    pub(in crate::input::runtime) fn status(&self) -> InputEventQueueStatus {
        InputEventQueueStatus {
            retained_events: self.events.len() as u32,
            coalesced_events: self.coalesced_events,
        }
    }
}
