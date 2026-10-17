use std::time::Duration;

use thiserror::Error;

/// Stable identity for one World-local fixed simulation step.
///
/// A tick index is only meaningful together with the World generation and the
/// fixed clock epoch that produced it.
/// 身份由所属 World 代际、固定域配置 epoch 和提交序号共同解释；不是全局唯一时间戳。
/// abort 后重试可沿用同一身份，不能仅凭取得此值断言该步已提交。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimulationTickId {
    /// 发放待提交步时由调用方捕获的 World 代际。
    world_generation: u64,
    /// 发放时固定时钟的配置代际，不是采样来源版本。
    fixed_epoch: u64,
    /// 下一次成功提交的固定步序号，生产者以饱和加法生成。
    tick_index: u64,
}

impl SimulationTickId {
    /// 直接组合三段身份，不查询 World 或验证单调性；生产者须先完成步准入。
    pub(crate) const fn new(world_generation: u64, fixed_epoch: u64, tick_index: u64) -> Self {
        Self {
            world_generation,
            fixed_epoch,
            tick_index,
        }
    }

    pub const fn world_generation(self) -> u64 {
        self.world_generation
    }

    pub const fn fixed_epoch(self) -> u64 {
        self.fixed_epoch
    }

    pub const fn tick_index(self) -> u64 {
        self.tick_index
    }
}

/// A non-cloneable capability for one begun but uncommitted fixed step.
/// 控制器登记 active 身份后交给调度器；令牌本身无析构结算，须显式 commit 或 abort。
/// WorldDriver 的持有守卫另在 Drop 中尝试 abort；它不保证撤销已运行系统的副作用。
#[derive(Debug)]
pub(crate) struct WorldFixedStep {
    /// 已获准但尚未提交的固定步身份。
    id: SimulationTickId,
    /// 本步成功提交时应推进的固定持续时间。
    timestep: Duration,
    /// 本步成功提交后的预计累计时长，尚非权威当前值。
    elapsed: Duration,
}

impl WorldFixedStep {
    /// 直接保存 `id`、`timestep` 与预计 `elapsed`；活动步与欠债检查由控制器先完成。
    pub(crate) const fn new(id: SimulationTickId, timestep: Duration, elapsed: Duration) -> Self {
        Self {
            id,
            timestep,
            elapsed,
        }
    }

    pub const fn id(&self) -> SimulationTickId {
        self.id
    }

    pub const fn timestep(&self) -> Duration {
        self.timestep
    }

    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

/// Rejection from a World-local fixed step state transition.
/// 控制器拒绝状态转换，或 Level 拒绝提交到已替换 World；错误字段保存实际比较的身份。
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub(crate) enum WorldFixedStepError {
    /// `active` 尚未结算，不能再次 begin。
    #[error("fixed step {active:?} is already active")]
    ActiveStep { active: SimulationTickId },
    /// 本外帧计划的剩余提交名额为零。
    #[error("no fixed-step budget remains for this outer frame")]
    BudgetExhausted,
    /// 当前债务不足一个完整步长，无法开始或提交。
    #[error("fixed-step debt is smaller than the requested timestep")]
    InsufficientDebt,
    /// 提交的 `submitted` 与当前 `active` 身份不同。
    #[error("fixed step {submitted:?} does not match the active step {active:?}")]
    ActiveStepMismatch {
        active: SimulationTickId,
        submitted: SimulationTickId,
    },
    /// `submitted` 没有对应的活动步，亦包括重复结算。
    #[error("fixed step {submitted:?} has no active transaction")]
    NoActiveStep { submitted: SimulationTickId },
    /// Level 提交时捕获的 `expected` 与当前 World 的 `actual` 不同。
    #[error("World generation changed during fixed step: expected {expected}, found {actual}")]
    WorldGenerationChanged { expected: u64, actual: u64 },
}
