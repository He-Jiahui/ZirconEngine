//! 在启用配置下有界保留未经过帧队列合并的原始输入记录。
//!
//! # 调用关系
//! [`crate::input::DefaultInputManager`] 在 `submit_event` 内先更新输入状态，再调用 [`InputEventRecorder::record`]，最后把事件移入帧事件缓冲。管理器转发配置、状态查询和记录排空；消费者可通过 [`crate::input::InputRecordingFrame::capture_from_manager`] 一并取得记录和录制状态。
//!
//! # 状态与生命周期
//! 默认关闭。启用且容量非零时每次记录各自保存事件副本，FIFO 满载或缩容时从队首淘汰最旧记录；启用且零容量时只推进序号并累计丢弃。禁用或从禁用切换为启用会重置队列、序号和丢弃计数，持续启用时保持当前周期并按新容量裁剪。
//!
//! 本记录器与帧内瞬时事件缓冲独立：管理器的帧边界不会清空记录，帧事件合并也不改写记录。[`InputEventRecorder::drain`] 转移记录，但保留配置、序号和丢弃计数。
//!
//! # 参考
//! [`InputEventRecord`]、[`InputEventRecordingConfig`]、[`InputEventRecordingStatus`]
use std::collections::VecDeque;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::input::{
    InputEvent, InputEventRecord, InputEventRecordingConfig, InputEventRecordingStatus,
};

// 可选的原始事件保留器独立于帧事件合并；容量丢弃数量传给回放完整性判断。
/// 按记录调用顺序保存原始输入，并报告当前录制周期的容量丢弃。
///
/// # 构造与使用
/// 默认配置为关闭，容量为 [`crate::input::DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`]，记录为空、丢弃计数和序号游标均为零。运行时输入状态持有本记录器，[`crate::input::DefaultInputManager`] 在自己的状态锁内访问它；本类型的可变方法借用记录器。
///
/// # 状态与生命周期
/// 启用后每次记录先饱和推进序号，零容量也消耗序号并累计丢弃；首次分配的序号为一，达到 `u64::MAX` 后可能重复。启用且容量非零时保存事件副本，不执行帧队列合并；持续启用的配置更新保留当前周期，容量不足时淘汰旧记录。
///
/// 排空只转移记录，配置、序号和丢弃计数继续存在。关闭记录会保存传入的关闭配置，并清空队列、序号和丢弃计数；从关闭切换为启用也重置这三个周期状态。管理器的帧边界不会清空本记录器。
///
/// # 参考
/// [`InputEventRecordingStatus`]、[`Self::configure`]、[`Self::drain`]、[`crate::input::InputRecordingFrame::capture_from_manager`]。捕获完整性同时取决于启用状态与累计丢弃数。
#[derive(Debug, Default)]
pub(in crate::input::runtime) struct InputEventRecorder {
    /// 当前录制开关和以记录条目数为单位的 `u32` 容量上限；零容量允许启用，但不保留记录。
    /// 关闭时 [`Self::record`] 不改变序号或丢弃计数；[`Self::configure`] 保存传入配置并处理周期状态重置或容量裁剪。
    config: InputEventRecordingConfig,
    /// 按 [`Self::record`] 调用先后保存的 FIFO，队首最旧；满载写入与缩容从队首淘汰，排空转移全部条目。
    /// 队列不按时间戳或序号排序；序号饱和后可重复，墙钟回退不会改变 FIFO 顺序。
    records: VecDeque<InputEventRecord>,
    /// 自当前启用周期以来因零容量、满载或缩容而丢弃的记录数，使用饱和加法。
    discarded_records: u64,
    /// 已推进的序号游标；默认及周期重置时为零，启用的 [`Self::record`] 先饱和加一再用于新记录，因此首次序号为一。
    /// 零容量也推进游标，排空不重置它；达到 `u64::MAX` 后后续记录可具有相同序号。
    next_sequence: u64,
}

impl InputEventRecorder {
    // 停止或重新开启记录时重置序列；持续记录时缩减容量会保留最新记录并累计丢弃量。
    /// 保存新的录制配置，并重置周期状态或按容量裁剪已有记录。
    ///
    /// # 参数
    /// `config` 按值传入；`enabled` 控制是否尝试记录，`capacity` 是 `u32` 记录条目数上限，零也有效。
    ///
    /// # 状态与生命周期
    /// 关闭配置被原样保存，同时清空队列、序号游标和丢弃计数。从关闭切换为启用也先重置这三个状态，随后保存新配置。持续启用时保持序号和累计丢弃数，缩容淘汰队首超额记录并饱和增加丢弃计数；增容或容量不变时保留现有记录。
    ///
    /// 启用且容量为零时裁剪掉当前全部记录，后续记录只推进序号并累计丢弃。排空不结束配置周期。
    ///
    /// # 返回
    /// 返回 `()`，配置保存及必要的裁剪已经同步完成；没有单独的错误返回值，后续 [`Self::record`] 使用新配置。
    ///
    /// # 参考
    /// [`InputEventRecordingConfig`]、[`Self::status`]、[`Self::drain`]
    pub(in crate::input::runtime) fn configure(&mut self, config: InputEventRecordingConfig) {
        if !config.enabled {
            self.config = config;
            self.records.clear();
            self.discarded_records = 0;
            self.next_sequence = 0;
            return;
        }

        if !self.config.enabled {
            self.records.clear();
            self.discarded_records = 0;
            self.next_sequence = 0;
        }
        self.config = config;
        trim_records_to_capacity(
            &mut self.records,
            config.capacity as usize,
            &mut self.discarded_records,
        );
    }

    /// 按启用配置尝试保存一个原始事件，并累计容量导致的丢弃。
    ///
    /// # 参数
    /// `event` 是本次原始提交事件的共享借用，不转移调用者的事件所有权。实际保留记录时克隆该事件；关闭或零容量时不克隆，也不采样时间戳。
    ///
    /// # 状态与生命周期
    /// 关闭时立即返回，不改变记录器。启用时先令序号游标饱和加一；零容量随后只令丢弃计数饱和加一。容量非零且队列已满时先淘汰队首并累计一次丢弃，再把新记录追加到队尾。
    ///
    /// 保留记录的序号使用此次推进后的值，首次为一，饱和后可能重复。时间戳采样墙钟相对 UNIX epoch 的毫秒数并转换为 `u64`；早于 epoch 时回退为零，超出 `u64` 范围的转换会截断。墙钟可回退，时间戳不保证单调，也不决定 FIFO 顺序。
    ///
    /// # 返回
    /// 返回 `()`，本次尝试已经同步完成；没有单独的保留回执或错误返回值。关闭或零容量不会保留记录，满载写入还会淘汰旧记录，调用者可通过 [`Self::status`] 观察状态。
    ///
    /// # 参考
    /// [`InputEventRecord`]、[`Self::configure`]、[`Self::drain`]
    pub(in crate::input::runtime) fn record(&mut self, event: &InputEvent) {
        if !self.config.enabled {
            return;
        }

        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.config.capacity == 0 {
            self.discarded_records = self.discarded_records.saturating_add(1);
            return;
        }
        if self.records.len() >= self.config.capacity as usize {
            self.records.pop_front();
            self.discarded_records = self.discarded_records.saturating_add(1);
        }
        self.records.push_back(InputEventRecord {
            sequence: self.next_sequence,
            timestamp_millis: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            event: event.clone(),
        });
    }

    /// 返回当前录制配置与保留、丢弃计数的即时快照。
    ///
    /// # 返回
    /// [`InputEventRecordingStatus`] 的 `enabled` 和 `capacity` 来自当前配置；`retained_records` 是当前 FIFO 长度转换为 `u32` 的结果，正常配置和记录路径将长度约束在 `u32` 容量上限内。`discarded_records` 是当前启用周期的累计饱和丢弃计数。
    ///
    /// # 生命周期
    /// 本方法不修改或排空记录器，快照只代表调用瞬间。[`Self::drain`] 后保留数归零，丢弃计数仍保留；关闭或从关闭切换为启用时重置周期计数。
    ///
    /// # 参考
    /// [`Self::configure`]、[`Self::record`]、[`Self::drain`]
    pub(in crate::input::runtime) fn status(&self) -> InputEventRecordingStatus {
        InputEventRecordingStatus {
            enabled: self.config.enabled,
            capacity: self.config.capacity,
            retained_records: self.records.len() as u32,
            discarded_records: self.discarded_records,
        }
    }

    /// 按 FIFO 的记录调用顺序排空当前保留记录，并转移其所有权。
    ///
    /// # 返回
    /// 返回独立拥有 [`InputEventRecord`] 的向量，队首旧记录在前；不按时间戳或序号重新排序。空向量表示当前没有保留记录，不能据此判断是否曾有提交或容量丢弃。
    ///
    /// # 调用时机与生命周期
    /// 管理器的记录消费或帧捕获入口转发本操作。队列同步变空，配置、序号游标和累计丢弃计数保持不变；配置仍启用时，后续记录继续进入同一周期。没有后续记录时再次调用返回空向量。
    ///
    /// [`crate::input::DefaultInputManager`] 的 `drain_event_records_with_status` 在同一个状态锁内排空后读取状态；返回状态的保留数因此为零，累计丢弃数仍可用于完整性判断。
    ///
    /// # 参考
    /// [`Self::status`]、[`Self::record`]、[`crate::input::InputRecordingFrame::capture_from_manager`]
    pub(in crate::input::runtime) fn drain(&mut self) -> Vec<InputEventRecord> {
        self.records.drain(..).collect()
    }
}

/// 裁剪 FIFO 队首的超额记录，并累计本次容量丢弃数。
///
/// # 参数
/// `records` 可变借用按记录调用顺序排列的队列；只移除队首超额条目，保留最新记录。`capacity` 是以记录条目数为单位的 `usize` 上限，零会移除全部记录。`discarded_records` 可变借用累计丢弃计数，以此次移除数量饱和增加。
///
/// # 返回与状态
/// 返回 `()`，队列已裁剪到不超过容量；原长度已满足容量时队列和计数保持不变。超额项一次批量移除，计数随后更新一次；本函数不改变录制开关或序号，也没有单独的错误返回值。
///
/// # 参考
/// [`InputEventRecorder::configure`]、[`InputEventRecord`]
fn trim_records_to_capacity(
    records: &mut VecDeque<InputEventRecord>,
    capacity: usize,
    discarded_records: &mut u64,
) {
    let excess = records.len().saturating_sub(capacity);
    if excess == 0 {
        return;
    }

    drop(records.drain(..excess));
    *discarded_records = discarded_records.saturating_add(excess as u64);
}

#[cfg(test)]
#[path = "tests/recorder.rs"]
mod tests;
