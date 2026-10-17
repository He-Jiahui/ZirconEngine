use std::sync::MutexGuard;
use std::time::Duration;

use crate::core::framework::time::{
    MonotonicReal, Time, TimePolicy, TimePolicyError, TimePolicyTransaction,
};

use super::super::frame_clock::{
    ClockDiscontinuity, FrameClock, FrameClockRebaseCause, FrameClockRebaseReceipt,
};
use super::super::time::{
    FrameTimeDiscontinuity, FrameTimeSnapshot, RuntimeTimeAuthority, TimePolicyReceipt,
    TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};
use super::CoreHandle;

impl CoreHandle {
    /// Returns the outer monotonic frame clock. World-derived clocks belong to Levels.
    /// 在时间锁内复制真实时钟值；返回后可独立保留，后续外帧不会更新该副本。
    pub fn real_time(&self) -> Time<MonotonicReal> {
        self.lock_time().real()
    }

    /// Returns the default policy used when a new Level is created.
    /// 复制供新建 Level 使用的默认策略；已有 Level 的即时配置应向该 Level 查询。
    pub fn time_policy(&self) -> TimePolicy {
        self.lock_time().time_policy()
    }

    /// Returns the generation of the default policy for subsequently created Levels.
    /// 读取默认策略实际变化的饱和版本；不能用来判断已有 World 的配置是否相同。
    pub fn time_policy_generation(&self) -> u64 {
        self.lock_time().time_policy_generation()
    }

    /// Changes the default policy for subsequently created Levels.
    ///
    /// Existing Levels retain their own timing policy and fixed debt. Live
    /// multi-World policy propagation requires an explicit Level transaction.
    /// `transaction` 先校验后应用；失败保留默认值和版本，成功返回可脱离锁保存的回执。
    /// 参考：Level 的独立策略事务另有活动步与欠债准入，不能以此默认值提交替代。
    pub fn apply_time_policy(
        &self,
        transaction: TimePolicyTransaction,
    ) -> Result<TimePolicyReceipt, TimePolicyError> {
        self.lock_time().apply_time_policy(transaction)
    }

    /// 用宿主提供的帧增量推进外层时间，供离线驱动或测试生成世界预算。
    /// 同一帧只应选择此入口或 [`Self::tick_time`]，避免重复推进。
    /// `real_delta` 原样记为真实增量；`max_fixed_steps` 原样交给各 World 的正常步计划。
    /// 返回外帧快照，不采样或重基线帧时钟；宿主须安排与采样入口之间的切换。
    pub fn advance_time_by(&self, real_delta: Duration, max_fixed_steps: u32) -> FrameTimeSnapshot {
        self.advance_time_by_with_discontinuity(real_delta, max_fixed_steps, None)
    }

    /// 从帧时钟采样并推进一次外层帧；重基准信息随快照交给会话与世界。
    ///
    /// # 参数与返回
    ///
    /// `max_fixed_steps` 是该外帧的正常固定步预算；返回快照可复用给多个 Level。
    /// 采样锁与时间锁依次取得，诊断另行记录；宿主负责按外帧顺序驱动本入口。
    pub fn tick_time(&self, max_fixed_steps: u32) -> FrameTimeSnapshot {
        let frame_tick = self.lock_frame_clock().tick();
        self.advance_time_by_with_discontinuity(
            frame_tick.delta(),
            max_fixed_steps,
            frame_tick
                .rebase()
                .map(FrameTimeDiscontinuity::FrameClockRebased),
        )
    }

    // 在时间锁内一次生成快照，释放后才写诊断；不把两个存储的观察承诺为同一原子操作。
    fn advance_time_by_with_discontinuity(
        &self,
        raw_real_delta: Duration,
        max_fixed_steps: u32,
        discontinuity: Option<FrameTimeDiscontinuity>,
    ) -> FrameTimeSnapshot {
        let snapshot = {
            let mut time = self.lock_time();
            time.advance_by_with_discontinuity(raw_real_delta, max_fixed_steps, discontinuity)
        };
        record_time_diagnostics(self, snapshot);
        snapshot
    }

    /// 会话激活完成后重建采样基线；回执由下一次采样取走，不推进真实或 World 时钟。
    pub(crate) fn rebase_frame_clock(&self) -> FrameClockRebaseReceipt {
        self.lock_frame_clock()
            .rebase_for(FrameClockRebaseCause::SessionActivationCompleted)
    }

    /// 宿主恢复或时钟跳变后重基准下一帧采样，避免把停顿时间计入模拟增量。
    /// `discontinuity` 记录重基线原因；返回凭据并留待下次采样转交，重复重基线保留最新凭据。
    /// 此处只调整后续采样基线，不清理已有 World 的时间、欠债或插值历史。
    pub fn submit_clock_discontinuity(
        &self,
        discontinuity: ClockDiscontinuity,
    ) -> FrameClockRebaseReceipt {
        self.lock_frame_clock()
            .rebase_for(FrameClockRebaseCause::ClockDiscontinuity(discontinuity))
    }

    fn lock_time(&self) -> MutexGuard<'_, RuntimeTimeAuthority> {
        self.inner
            .time
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_frame_clock(&self) -> MutexGuard<'_, FrameClock> {
        self.inner
            .frame_clock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

// 时间状态提交之后记录外帧计数；零增量提前返回，保留已有时长/FPS 样本。
fn record_time_diagnostics(handle: &CoreHandle, snapshot: FrameTimeSnapshot) {
    let frame_index = snapshot.outer_frame_index();
    let real_delta_seconds = snapshot.raw_real_delta().as_secs_f64();
    let mut diagnostics = handle.lock_diagnostics();

    diagnostics.record_static(
        TIME_FRAME_COUNT_DIAGNOSTIC,
        frame_index,
        frame_index as f64,
        Some("frame"),
        &["time", "frame"],
    );
    if real_delta_seconds == 0.0 {
        return;
    }
    diagnostics.record_static(
        TIME_FRAME_TIME_DIAGNOSTIC,
        frame_index,
        real_delta_seconds * 1_000.0,
        Some("ms"),
        &["time", "frame"],
    );
    diagnostics.record_static(
        TIME_FPS_DIAGNOSTIC,
        frame_index,
        1.0 / real_delta_seconds,
        Some("hz"),
        &["time", "frame"],
    );
}

#[cfg(test)]
#[path = "tests/time.rs"]
mod tests;
