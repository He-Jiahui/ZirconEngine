use std::time::Duration;

use crate::core::framework::time::{ClockDomainId, ClockDomainStamp, FixedStepPlan};
use crate::core::{FrameTimeDiscontinuity, FrameTimeSnapshot};

/// Immutable virtual/fixed timing work accepted by one World for one outer frame.
///
/// The outer frame's monotonic time remains owned by `CoreRuntime`; this value
/// owns the World-local derived clocks, pause state, and fixed-step debt.
/// 此处“拥有”指保存派生观测值；实际可变权威仍由所属 Level 的控制器持有。
///
/// # 观察与生命周期
///
/// 固定计划是在本帧开始提交前生成的预测，后续失败、提交或中止不回写此副本。
/// 该值不携带 World 身份；驱动器另行捕获代际，构造各系统的时钟上下文。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTimeSnapshot {
    /// 已由该控制器接受的外帧序号。
    outer_frame_index: u64,
    /// 与外帧一致的未缩放真实增量。
    raw_real_delta: Duration,
    /// 与外帧一致的真实累计时长。
    real_elapsed: Duration,
    /// 本次按该 World 暂停、倍率和上限采样的虚拟增量。
    virtual_delta: Duration,
    /// 本次推进后该 World 的虚拟累计时长。
    virtual_elapsed: Duration,
    /// 本次推进采样的独立暂停标志。
    virtual_time_paused: bool,
    /// 本次虚拟推进实际采用的倍率缓存。
    virtual_effective_speed: f64,
    /// 生成快照时该控制器的策略版本。
    time_policy_generation: u64,
    /// 直接保留外帧携带的重基线凭据。
    discontinuity: Option<FrameTimeDiscontinuity>,
    /// 按捕获时的欠债和名额生成的未执行计划。
    fixed_step_plan: FixedStepPlan,
    /// 直接保留外帧的真实域与来源版本。
    real_clock_domain_stamp: ClockDomainStamp,
    /// 本 World 虚拟域的配置与来源版本。
    virtual_clock_domain_stamp: ClockDomainStamp,
    /// 本 World 固定域的配置与来源版本。
    fixed_clock_domain_stamp: ClockDomainStamp,
}

impl WorldTimeSnapshot {
    /// 将 `outer` 的真实证据与已准入的 World 派生观测组合成独立快照。
    /// 参数包含本次虚拟样本、策略版本、未执行固定计划与两个派生 stamp；此处只保存，不再校验。
    /// 调用方负责在外帧准入和派生推进之后捕获一致参数，返回值不借用活动时钟。
    pub(crate) fn new(
        outer: FrameTimeSnapshot,
        virtual_delta: Duration,
        virtual_elapsed: Duration,
        virtual_time_paused: bool,
        virtual_effective_speed: f64,
        time_policy_generation: u64,
        fixed_step_plan: FixedStepPlan,
        virtual_clock_domain_stamp: ClockDomainStamp,
        fixed_clock_domain_stamp: ClockDomainStamp,
    ) -> Self {
        Self {
            outer_frame_index: outer.outer_frame_index(),
            raw_real_delta: outer.raw_real_delta(),
            real_elapsed: outer.real_elapsed(),
            virtual_delta,
            virtual_elapsed,
            virtual_time_paused,
            virtual_effective_speed,
            time_policy_generation,
            discontinuity: outer.discontinuity(),
            fixed_step_plan,
            real_clock_domain_stamp: outer.real_clock_domain_stamp(),
            virtual_clock_domain_stamp,
            fixed_clock_domain_stamp,
        }
    }

    pub const fn outer_frame_index(self) -> u64 {
        self.outer_frame_index
    }

    pub const fn raw_real_delta(self) -> Duration {
        self.raw_real_delta
    }

    pub const fn real_elapsed(self) -> Duration {
        self.real_elapsed
    }

    pub const fn virtual_delta(self) -> Duration {
        self.virtual_delta
    }

    pub const fn virtual_elapsed(self) -> Duration {
        self.virtual_elapsed
    }

    pub const fn virtual_time_paused(self) -> bool {
        self.virtual_time_paused
    }

    pub const fn virtual_effective_speed(self) -> f64 {
        self.virtual_effective_speed
    }

    pub const fn time_policy_generation(self) -> u64 {
        self.time_policy_generation
    }

    pub const fn discontinuity(self) -> Option<FrameTimeDiscontinuity> {
        self.discontinuity
    }

    /// 读取生成快照时的计划副本；实际剩余债务及已提交端点应向 Level 重新查询。
    pub const fn fixed_step_plan(self) -> FixedStepPlan {
        self.fixed_step_plan
    }

    pub const fn real_clock_domain_stamp(self) -> ClockDomainStamp {
        self.real_clock_domain_stamp
    }

    /// 返回本快照保留的真实、虚拟或固定域 stamp；其他域为 None，不临时创建时钟。
    pub const fn clock_domain_stamp(self, domain: ClockDomainId) -> Option<ClockDomainStamp> {
        match domain {
            ClockDomainId::MonotonicReal => Some(self.real_clock_domain_stamp),
            ClockDomainId::WorldVirtual => Some(self.virtual_clock_domain_stamp),
            ClockDomainId::WorldFixed => Some(self.fixed_clock_domain_stamp),
            ClockDomainId::WallUtc
            | ClockDomainId::Input
            | ClockDomainId::Render
            | ClockDomainId::Audio
            | ClockDomainId::Network
            | ClockDomainId::Media
            | ClockDomainId::EditorPreview => None,
        }
    }
}
