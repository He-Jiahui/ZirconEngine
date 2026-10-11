use std::time::Duration;

use thiserror::Error;

use super::{Fixed, Virtual};

/// 供 Runtime 默认配置和各 World 独立采纳的虚拟时间、固定步策略值。
///
/// # 责任与生命周期
///
/// 本值可拷贝，不持有时钟。构造与 `with_*` 方法可暂存无效值；提交前由
/// [validate](Self::validate) 或 [TimePolicyTransaction] 检查值域。
/// Runtime 的提交只更新后续新建 World 的默认值，已有 World 通过自己的事务入口改变配置。
///
/// # 值的含义
///
/// `virtual_max_delta` 限制变速后每帧的虚拟增量；`virtual_relative_speed` 是真实到虚拟时间的倍率。
/// 倍率为零可抑制推进，但不会设置 World 的独立暂停标志；`fixed_timestep` 用来解释固定步欠债。
/// 运行时的活动步、待偿债务及代际更新由时钟所有者检查和维护。
/// Atomically applied configuration for the runtime-owned virtual and fixed clocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimePolicy {
    virtual_max_delta: Duration,
    virtual_relative_speed: f64,
    fixed_timestep: Duration,
}

impl Default for TimePolicy {
    fn default() -> Self {
        Self::new(
            Virtual::default().max_delta(),
            Virtual::default().relative_speed_f64(),
            Fixed::default().timestep(),
        )
    }
}

impl TimePolicy {
    /// 构造一组待校验的时间策略值。
    ///
    /// # 参数
    ///
    /// - `virtual_max_delta`：变速后虚拟增量的非零上限。
    /// - `virtual_relative_speed`：有限且非负的倍率；零值与独立暂停状态分别管理。
    /// - `fixed_timestep`：每个固定步提交时推进的非零持续时间。
    ///
    /// # 返回与约束
    ///
    /// 返回独立的值副本，不做上述校验或应用；参数不合法时也能构造成功。
    /// 应先 [validate](Self::validate)，或交由 [TimePolicyTransaction] 的准备流程处理。
    pub const fn new(
        virtual_max_delta: Duration,
        virtual_relative_speed: f64,
        fixed_timestep: Duration,
    ) -> Self {
        Self {
            virtual_max_delta,
            virtual_relative_speed,
            fixed_timestep,
        }
    }

    pub const fn virtual_max_delta(self) -> Duration {
        self.virtual_max_delta
    }

    pub const fn virtual_relative_speed(self) -> f64 {
        self.virtual_relative_speed
    }

    pub const fn fixed_timestep(self) -> Duration {
        self.fixed_timestep
    }

    pub const fn with_virtual_max_delta(mut self, virtual_max_delta: Duration) -> Self {
        self.virtual_max_delta = virtual_max_delta;
        self
    }

    pub const fn with_virtual_relative_speed(mut self, virtual_relative_speed: f64) -> Self {
        self.virtual_relative_speed = virtual_relative_speed;
        self
    }

    /// 准备使用新固定步长的策略副本。
    ///
    /// 此操作只改变请求值，不清空或重新解释现有欠债。实际 World 提交时，
    /// 只要有活动步就拒绝请求；步长确实变化时，还要求旧欠债为零，包括不足一整步的余数。
    /// 失败不会应用其他字段；纯虚拟配置变化不要求固定欠债为零。
    pub const fn with_fixed_timestep(mut self, fixed_timestep: Duration) -> Self {
        self.fixed_timestep = fixed_timestep;
        self
    }

    /// 检查策略值是否可交给时钟所有者。
    ///
    /// # 返回与错误
    ///
    /// 成功返回 `()`；依次检查非零虚拟上限、有限倍率、非负倍率和非零固定步长，
    /// 遇到第一项不满足的值返回对应 [TimePolicyError]。零倍率和负零均可通过。
    ///
    /// # 验证边界
    ///
    /// 不读取或修改 Runtime、World；成功不代表当前 World 没有活动固定步或旧欠债，
    /// 这些状态条件必须在实际提交时重新检查。
    pub fn validate(self) -> Result<(), TimePolicyError> {
        if self.virtual_max_delta.is_zero() {
            return Err(TimePolicyError::VirtualMaxDeltaZero);
        }
        if !self.virtual_relative_speed.is_finite() {
            return Err(TimePolicyError::VirtualRelativeSpeedNotFinite);
        }
        if self.virtual_relative_speed < 0.0 {
            return Err(TimePolicyError::VirtualRelativeSpeedNegative);
        }
        if self.fixed_timestep.is_zero() {
            return Err(TimePolicyError::FixedTimestepZero);
        }
        Ok(())
    }
}

/// 交给 Runtime 或 World 的时间策略请求值。
///
/// # 提交流程
///
/// [new](Self::new) 仅封装请求，[prepare](Self::prepare) 检查值域并返回策略副本。
/// 成功准备不预留状态、不生成提交回执，也不改变时钟；调用者仍须交给具体所有者。
/// Runtime 管理新 World 的默认值，World 在应用前另外检查活动步及步长变化时的欠债。
/// 请求可独立拷贝；它不绑定某个 World 身份，也不证明之后的状态仍满足提交条件。
/// A requested time-policy change that must validate before the runtime mutates its clocks.
/// 时钟策略的提交请求；先校验值，再由 Runtime/World 权威检查活动固定步和待偿欠债后原子应用。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimePolicyTransaction {
    requested: TimePolicy,
}

impl TimePolicyTransaction {
    /// 封装一次时间策略请求。
    ///
    /// # 参数与返回
    ///
    /// `requested` 可以尚未通过值域校验；返回保存该值的可拷贝请求。
    /// 构造不报错、不应用配置，后续用 [prepare](Self::prepare) 或所有者的事务入口处理。
    pub const fn new(requested: TimePolicy) -> Self {
        Self { requested }
    }

    pub const fn requested(self) -> TimePolicy {
        self.requested
    }

    pub fn validate(self) -> Result<(), TimePolicyError> {
        self.requested.validate()
    }

    /// 校验请求值并取出供时钟所有者使用的策略。
    ///
    /// # 返回与错误
    ///
    /// 成功返回 [TimePolicy] 副本；失败透传 [TimePolicy::validate] 的首个值域错误。
    /// 此阶段不检查 World 的活动固定步或待偿欠债，也不修改任何配置。
    ///
    /// # 后续动作
    ///
    /// 调用者须将结果交给具体所有者完成状态检查与提交；得到值不能当作提交回执，
    /// 也不保证稍后的 World 应用一定成功。
    /// Prepares the requested values without mutating a runtime clock authority.
    pub fn prepare(self) -> Result<TimePolicy, TimePolicyError> {
        self.validate()?;
        Ok(self.requested)
    }
}

/// 时间策略的值域拒绝，或 World 应用时的状态冲突。
///
/// # 发生边界
///
/// [TimePolicy::validate] 只产生前四种值域错误。World 在所有写入前还检查活动固定步，
/// 以及改变固定步长时是否存在旧欠债；欠债余数也会触发拒绝。
/// 活动步应先提交或中止；欠债冲突应保留原策略，不能把新步长直接用于旧欠债。
/// Runtime 更新默认策略不读取已有 World 的这些状态。
/// Typed rejection for a requested runtime time policy.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum TimePolicyError {
    #[error("virtual time max delta must be non-zero")]
    VirtualMaxDeltaZero,
    #[error("virtual time relative speed must be finite")]
    VirtualRelativeSpeedNotFinite,
    #[error("virtual time relative speed must be non-negative")]
    VirtualRelativeSpeedNegative,
    #[error("fixed timestep must be non-zero")]
    FixedTimestepZero,
    #[error("time policy cannot change while a fixed step is active")]
    FixedStepActive,
    #[error("fixed timestep cannot change while {remaining:?} of fixed debt is pending")]
    FixedStepDebtPending { remaining: Duration },
}
