use std::time::Duration;

use crate::core::framework::time::{
    ClockDomainId, ClockDomainStamp, MonotonicReal, Time, TimePolicy, TimePolicyError,
    TimePolicyTransaction,
};

use super::frame_clock::FrameClockRebaseReceipt;

#[path = "time/product_policy.rs"]
mod product_policy;

pub use product_policy::{ProductTimePolicies, ProductTimePolicyDigest};

/// Diagnostic path for total real-time frames advanced by the runtime.
/// 外帧累计推进序号的诊断键；零增量外帧也会记录。
pub const TIME_FRAME_COUNT_DIAGNOSTIC: &str = "time.frame_count";
/// Diagnostic path for real frame duration measured in milliseconds.
/// 非零外帧真实增量的毫秒诊断键；零增量不会覆盖旧时长记录。
pub const TIME_FRAME_TIME_DIAGNOSTIC: &str = "time.frame_time";
/// Diagnostic path for frames per second derived from real frame duration.
/// 非零真实帧时长倒数的诊断键；该值与 World 的暂停或倍率无关。
pub const TIME_FPS_DIAGNOSTIC: &str = "time.fps";

/// Core-owned outer-frame clock and defaults for subsequently created Worlds.
///
/// Virtual/fixed simulation time is deliberately absent: each `LevelSystem`
/// owns those derived clocks, their debt, and their commit boundary.
/// 在 CoreHandle 的时间锁内维护外帧与未来 World 的默认配置；观察副本不会跟随更新。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RuntimeTimeAuthority {
    real: Time<MonotonicReal>,
    /// 仅供后续 Level 初始化采纳，已有 Level 继续使用自己的策略。
    default_world_time_policy: TimePolicy,
    /// 默认策略实际改变时饱和递增；不是外帧序号或 World 配置版本。
    policy_generation: u64,
}

impl Default for RuntimeTimeAuthority {
    fn default() -> Self {
        Self {
            real: Time::<MonotonicReal>::default(),
            default_world_time_policy: TimePolicy::default(),
            policy_generation: 0,
        }
    }
}

impl RuntimeTimeAuthority {
    pub(crate) const fn real(&self) -> Time<MonotonicReal> {
        self.real
    }

    pub(crate) const fn time_policy(&self) -> TimePolicy {
        self.default_world_time_policy
    }

    pub(crate) const fn time_policy_generation(&self) -> u64 {
        self.policy_generation
    }

    /// 准备并采纳 `transaction` 的默认配置，返回旧值、新值和当前版本回执。
    /// 值域错误在写入前返回；相等配置不递增版本，也不广播到已存在的 World。
    pub(crate) fn apply_time_policy(
        &mut self,
        transaction: TimePolicyTransaction,
    ) -> Result<TimePolicyReceipt, TimePolicyError> {
        let applied = transaction.prepare()?;
        let previous = self.default_world_time_policy;
        let changed = applied != previous;

        if changed {
            self.default_world_time_policy = applied;
            self.policy_generation = self.policy_generation.saturating_add(1);
        }

        Ok(TimePolicyReceipt {
            previous,
            applied,
            generation: self.policy_generation,
            changed,
        })
    }

    // 外帧重基线先更新真实时钟的来源代际，再生成供各 Level 消费的同一快照。
    /// 将已选定的 `raw_real_delta` 发布为一次外帧；零增量也推进序号。
    /// `fixed_step_budget` 原样进入快照，允许零；本层不派生 World 时钟或执行固定步。
    /// `discontinuity` 若含重基线回执，先复制来源版本；返回值是本次提交的独立观测。
    pub(crate) fn advance_by_with_discontinuity(
        &mut self,
        raw_real_delta: Duration,
        fixed_step_budget: u32,
        discontinuity: Option<FrameTimeDiscontinuity>,
    ) -> FrameTimeSnapshot {
        if let Some(FrameTimeDiscontinuity::FrameClockRebased(receipt)) = discontinuity {
            self.real
                .set_clock_domain_source_generation(receipt.generation());
        }
        self.real.advance_by(raw_real_delta);

        FrameTimeSnapshot {
            outer_frame_index: self.real.frame_index(),
            raw_real_delta,
            real_elapsed: self.real.elapsed(),
            fixed_step_budget,
            discontinuity,
            real_clock_domain_stamp: self.real.clock_domain_stamp(),
        }
    }
}

/// Immutable evidence for an accepted default World time-policy transaction.
/// 默认策略事务的值回执，可在时间锁外保留；相等请求也成功，但 `changed` 为假。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimePolicyReceipt {
    /// 本次请求前的默认策略。
    previous: TimePolicy,
    /// 本次通过准入的请求值；相等时可保留原配置的位模式，例如正负零。
    applied: TimePolicy,
    /// 提交后的默认策略版本，实际变化时饱和递增。
    generation: u64,
    /// 新旧策略按值比较是否不同。
    changed: bool,
}

impl TimePolicyReceipt {
    pub const fn previous(self) -> TimePolicy {
        self.previous
    }

    pub const fn applied(self) -> TimePolicy {
        self.applied
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn changed(self) -> bool {
        self.changed
    }
}

/// Typed discontinuity observed while preparing one outer-frame time snapshot.
/// 外帧采样边界的类型化凭据；本身不重置 World 的累计时间或固定债务。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameTimeDiscontinuity {
    /// 底层采样已重基线；来源代际随本外帧传给各 World。
    FrameClockRebased(FrameClockRebaseReceipt),
}

/// Immutable real-time input and fixed-step budget for one outer frame.
///
/// World-local virtual/fixed delta, pause state, debt, and clock stamps are
/// materialized later as `WorldTimeSnapshot` by the owning `LevelSystem`.
/// 在 Runtime 时间锁内生成的不可变副本，供多个 Level 消费同一外帧输入。
/// 读取不重新采样；来源 stamp 不携带 World 身份，也不证明某个 Level 已接受该外帧。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameTimeSnapshot {
    /// Runtime 的外帧推进序号，饱和递增，零增量也计入。
    outer_frame_index: u64,
    /// 宿主显式提供或帧时钟采得的真实增量，尚未按 World 倍率缩放或截断。
    raw_real_delta: Duration,
    /// 与本次外帧一同捕获的真实累计时长。
    real_elapsed: Duration,
    /// 交给各 World 正常固定步计划的预算值；显式暂停单步分支另行决策。
    fixed_step_budget: u32,
    /// 本次采样携带的重基线凭据，未携带时为 None。
    discontinuity: Option<FrameTimeDiscontinuity>,
    /// 本外帧的真实域类别与配置、来源代际观测。
    real_clock_domain_stamp: ClockDomainStamp,
}

impl FrameTimeSnapshot {
    pub const fn outer_frame_index(self) -> u64 {
        self.outer_frame_index
    }

    pub const fn raw_real_delta(self) -> Duration {
        self.raw_real_delta
    }

    /// Monotonic-real elapsed time captured atomically with this outer frame.
    pub const fn real_elapsed(self) -> Duration {
        self.real_elapsed
    }

    /// Maximum fixed steps the outer-frame owner permits each World to commit.
    /// 当前 World 的正常推进按此预算计划；暂停单步另取一个名额，其零预算优先级待明确。
    pub const fn fixed_step_budget(self) -> u32 {
        self.fixed_step_budget
    }

    pub const fn discontinuity(self) -> Option<FrameTimeDiscontinuity> {
        self.discontinuity
    }

    /// The shared monotonic source stamp for the outer frame.
    pub const fn real_clock_domain_stamp(self) -> ClockDomainStamp {
        self.real_clock_domain_stamp
    }

    /// Only the monotonic-real domain is owned by this outer-frame snapshot.
    pub const fn clock_domain_stamp(self, domain: ClockDomainId) -> Option<ClockDomainStamp> {
        match domain {
            ClockDomainId::MonotonicReal => Some(self.real_clock_domain_stamp),
            ClockDomainId::WorldVirtual
            | ClockDomainId::WorldFixed
            | ClockDomainId::WallUtc
            | ClockDomainId::Input
            | ClockDomainId::Render
            | ClockDomainId::Audio
            | ClockDomainId::Network
            | ClockDomainId::Media
            | ClockDomainId::EditorPreview => None,
        }
    }
}
