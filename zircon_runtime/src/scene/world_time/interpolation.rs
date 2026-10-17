use std::time::Duration;

use super::SimulationTickId;

/// One committed endpoint available to fixed-state interpolation consumers.
///
/// The initial World state is a committed baseline without a simulation tick.
/// It remains observable so a first committed fixed step can interpolate from
/// the initial state without inventing a tick identity.
/// 已提交时间端点的值证据，不持有物理状态对象；可在锁外保留。
/// 无 tick 也可能是策略变更或 World 替换清空历史后的当前基线，不保证 elapsed 为零。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedInterpolationState {
    /// 已提交步身份；尚无该历史端点身份时为空。
    simulation_tick: Option<SimulationTickId>,
    /// 此端点已提交的固定累计时长。
    elapsed: Duration,
}

impl FixedInterpolationState {
    /// 直接组合 `simulation_tick` 与 `elapsed`；调用方须保证两者属于同一端点。
    pub(crate) const fn new(simulation_tick: Option<SimulationTickId>, elapsed: Duration) -> Self {
        Self {
            simulation_tick,
            elapsed,
        }
    }

    pub const fn simulation_tick(self) -> Option<SimulationTickId> {
        self.simulation_tick
    }

    pub const fn elapsed(self) -> Duration {
        self.elapsed
    }
}

/// Immutable interpolation evidence derived from committed World-local fixed state.
///
/// `fraction` is the residual fixed debt modulo one timestep. It never exposes
/// a begun step and stays bounded even when the fixed-step admission cap leaves
/// multiple whole steps pending.
/// 拷贝两个已提交端点和捕获时的实际债务；后续 commit/abort 不会更新此副本。
/// 本值只提供时间证据，不执行插值，也不持有前后 World 状态。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedInterpolationContext {
    /// 前一已提交端点或历史为空时的当前基线。
    previous: FixedInterpolationState,
    /// 最近已提交端点，排除尚未 commit 的活动步。
    current: FixedInterpolationState,
    /// 捕获时的全部实际债务，可能含多个完整步。
    remaining_debt: Duration,
    /// 本次债务与比例所用的固定步长。
    timestep: Duration,
    /// 债务对步长取余后的 f32 比例，靠近一时可能舍入到 1.0。
    fraction: f32,
}

impl FixedInterpolationContext {
    /// 直接保存两个端点、债务、步长和比例，不验证关系或重新计算 fraction。
    /// 生产调用方从同一控制器观测生成参数；返回值不保留对控制器的借用。
    pub(crate) const fn new(
        previous: FixedInterpolationState,
        current: FixedInterpolationState,
        remaining_debt: Duration,
        timestep: Duration,
        fraction: f32,
    ) -> Self {
        Self {
            previous,
            current,
            remaining_debt,
            timestep,
            fraction,
        }
    }

    pub const fn previous(self) -> FixedInterpolationState {
        self.previous
    }

    pub const fn current(self) -> FixedInterpolationState {
        self.current
    }

    pub const fn remaining_debt(self) -> Duration {
        self.remaining_debt
    }

    pub const fn timestep(self) -> Duration {
        self.timestep
    }

    pub const fn fraction(self) -> f32 {
        self.fraction
    }
}
