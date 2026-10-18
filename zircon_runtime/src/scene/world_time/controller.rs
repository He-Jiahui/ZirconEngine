use std::time::Duration;

use crate::core::framework::time::{
    Fixed, Time, TimePolicy, TimePolicyError, TimePolicyTransaction, Virtual,
};
use crate::core::FrameTimeSnapshot;
use thiserror::Error;

use super::{
    FixedInterpolationContext, FixedInterpolationState, SimulationTickId, WorldFixedStep,
    WorldFixedStepError, WorldReplacementError, WorldReplacementTimeDisposition, WorldTimeSnapshot,
};

/// Mutable timing authority owned by exactly one `LevelSystem`.
///
/// It derives virtual and fixed time from the shared monotonic outer-frame
/// input, without sharing pause state, rate, or fixed debt with another World.
/// 由所属 Level 的锁串行访问；同一外帧可供多个控制器使用，它们各自维护推进与提交状态。
#[derive(Debug, PartialEq)]
pub(crate) struct WorldTimeController {
    /// 本 Level 的虚拟推进值、暂停与倍率配置。
    virtual_time: Time<Virtual>,
    /// 本 Level 的已提交固定时钟与全部待偿债务。
    fixed_time: Time<Fixed>,
    /// 本控制器配置实际改变时饱和递增的版本。
    policy_generation: u64,
    /// 本次已接受外帧计划中尚未成功提交的名额。
    fixed_step_budget_remaining: u32,
    /// 唯一已开始但尚未 commit 或 abort 的步身份。
    active_fixed_step: Option<SimulationTickId>,
    /// 插值历史中的前一个已提交步，基线或历史重置时为空。
    previous_committed_fixed_step: Option<SimulationTickId>,
    /// 前一提交端点的实际累计时长；历史重置时保存当前基线，不能由饱和时钟反推。
    previous_committed_fixed_elapsed: Duration,
    /// 插值历史中的最近已提交步，活动步不进入此字段。
    current_committed_fixed_step: Option<SimulationTickId>,
    /// 最近成功接受的外帧序号；拒绝帧不更新。
    last_outer_frame_index: Option<u64>,
    /// 已获准但尚未由有效外帧消费的一次暂停单步请求。
    single_step_requested: bool,
}

// Only concrete Copy clock observations cross preparation. This value owns no
// callbacks, World, resources, generic context, or active fixed-step capability.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PreparedWorldTimeReplacement {
    before: WorldTimeState,
    after: WorldTimeState,
}

impl PreparedWorldTimeReplacement {
    pub(crate) const fn before(self) -> WorldTimeState {
        self.before
    }

    pub(crate) const fn after(self) -> WorldTimeState {
        self.after
    }
}

impl Default for WorldTimeController {
    fn default() -> Self {
        Self::new(TimePolicy::default()).expect("default world time policy is valid")
    }
}

impl WorldTimeController {
    /// 先验证 `policy`，再创建零累计时长、零债务且无活动步的新权威。
    /// 无效策略返回原类型化错误；有效倍率在首次虚拟推进时采样为 effective_speed。
    pub fn new(policy: TimePolicy) -> Result<Self, TimePolicyError> {
        policy.validate()?;
        let mut virtual_time = Time::default();
        virtual_time.set_max_delta(policy.virtual_max_delta());
        virtual_time.set_relative_speed_f64(policy.virtual_relative_speed());
        let mut fixed_time = Time::default();
        fixed_time.set_timestep(policy.fixed_timestep());
        Ok(Self {
            virtual_time,
            fixed_time,
            policy_generation: 0,
            fixed_step_budget_remaining: 0,
            active_fixed_step: None,
            previous_committed_fixed_step: None,
            previous_committed_fixed_elapsed: Duration::ZERO,
            current_committed_fixed_step: None,
            last_outer_frame_index: None,
            single_step_requested: false,
        })
    }

    /// 复制锁外观察值；不暴露活动令牌或本帧剩余名额，副本不随提交更新。
    pub fn state(&self) -> WorldTimeState {
        WorldTimeState {
            virtual_time: self.virtual_time,
            fixed_time: self.fixed_time,
            policy_generation: self.policy_generation,
            last_outer_frame_index: self.last_outer_frame_index,
        }
    }

    /// Prepares concrete clock copies without changing this controller. The
    /// Level owner validates its World and checkpoint bindings separately.
    pub(crate) fn prepare_world_replacement(
        &self,
        disposition: WorldReplacementTimeDisposition,
        require_quiet: bool,
    ) -> Result<PreparedWorldTimeReplacement, WorldReplacementError> {
        if require_quiet {
            if let Some(active) = self.active_fixed_step {
                return Err(WorldReplacementError::FixedStepActive { active });
            }
            if self.fixed_step_budget_remaining != 0 {
                return Err(WorldReplacementError::FixedStepBudgetPending {
                    remaining: self.fixed_step_budget_remaining,
                });
            }
        }

        let before = self.state();
        let mut after = before;
        match disposition {
            WorldReplacementTimeDisposition::Preserve => {}
            WorldReplacementTimeDisposition::Reset { policy, paused } => {
                policy.validate()?;
                let mut virtual_time = Time::<Virtual>::default();
                virtual_time.set_max_delta(policy.virtual_max_delta());
                virtual_time.set_relative_speed_f64(policy.virtual_relative_speed());
                if paused {
                    virtual_time.pause();
                }
                let fixed_time = Time::<Fixed>::from_duration(policy.fixed_timestep());
                after.virtual_time = virtual_time;
                after.fixed_time = fixed_time;
            }
            WorldReplacementTimeDisposition::Restore(checkpoint) => {
                after = checkpoint;
            }
        }

        if !matches!(disposition, WorldReplacementTimeDisposition::Preserve) {
            let generation_floor = before.policy_generation.max(after.policy_generation);
            let generation = generation_floor.checked_add(1).ok_or(
                WorldReplacementError::PolicyGenerationExhausted {
                    current: generation_floor,
                },
            )?;

            let virtual_clock = after.virtual_time.clock_domain_stamp();
            let virtual_floor = before.virtual_time.clock_domain_stamp().epoch();
            after
                .virtual_time
                .try_rebase_clock_domain_epoch_after(virtual_floor)
                .ok_or(WorldReplacementError::ClockEpochExhausted {
                    clock: virtual_clock,
                    floor: virtual_floor,
                })?;
            let fixed_clock = after.fixed_time.clock_domain_stamp();
            let fixed_floor = before.fixed_time.clock_domain_stamp().epoch();
            after
                .fixed_time
                .try_rebase_clock_domain_epoch_after(fixed_floor)
                .ok_or(WorldReplacementError::ClockEpochExhausted {
                    clock: fixed_clock,
                    floor: fixed_floor,
                })?;

            // Restoring a World does not rewind the Runtime's accepted outer
            // frame or its source generation. Only World-local values rewind.
            after.virtual_time.set_clock_domain_source_generation(
                before.virtual_time.clock_domain_stamp().source_generation(),
            );
            after.fixed_time.set_clock_domain_source_generation(
                before.fixed_time.clock_domain_stamp().source_generation(),
            );
            after.policy_generation = generation;
            after.last_outer_frame_index = before.last_outer_frame_index;
        }
        Ok(PreparedWorldTimeReplacement { before, after })
    }

    /// This is the non-fallible clock half of the owner's pair publication.
    /// Legacy Preserve deliberately keeps active_fixed_step and its budget:
    /// the existing WorldDriver guard subsequently aborts that old capability.
    pub(crate) fn commit_world_replacement(&mut self, prepared: PreparedWorldTimeReplacement) {
        self.virtual_time = prepared.after.virtual_time;
        self.fixed_time = prepared.after.fixed_time;
        self.policy_generation = prepared.after.policy_generation;
        self.last_outer_frame_index = prepared.after.last_outer_frame_index;
        self.reset_after_world_replacement();
    }

    /// 从当前配置重建策略值，不包含暂停状态、有效倍率缓存或固定欠债。
    pub fn time_policy(&self) -> TimePolicy {
        TimePolicy::new(
            self.virtual_time.max_delta(),
            self.virtual_time.relative_speed_f64(),
            self.fixed_time.timestep(),
        )
    }

    /// 先准备 `transaction`，再检查本 World 的活动步和固定债务，全部准入后才写配置。
    ///
    /// # 返回与约束
    ///
    /// 活动步存在时拒绝；改变固定步长还要求全部债务为零，包括不足一步的余数。
    /// 成功返回旧/新配置及版本；仅实际改变的时钟更新 epoch，固定步长改变另清插值历史。
    pub fn apply_time_policy(
        &mut self,
        transaction: TimePolicyTransaction,
    ) -> Result<WorldTimePolicyReceipt, TimePolicyError> {
        let applied = transaction.prepare()?;
        if self.active_fixed_step.is_some() {
            return Err(TimePolicyError::FixedStepActive);
        }
        let previous = self.time_policy();
        if applied.fixed_timestep() != previous.fixed_timestep()
            && !self.fixed_time.overstep().is_zero()
        {
            return Err(TimePolicyError::FixedStepDebtPending {
                remaining: self.fixed_time.overstep(),
            });
        }
        let changed = applied != previous;
        if changed {
            let virtual_clock_changed = applied.virtual_max_delta() != previous.virtual_max_delta()
                || applied.virtual_relative_speed() != previous.virtual_relative_speed();
            let fixed_clock_changed = applied.fixed_timestep() != previous.fixed_timestep();
            if virtual_clock_changed {
                self.virtual_time.set_max_delta(applied.virtual_max_delta());
                self.virtual_time
                    .set_relative_speed_f64(applied.virtual_relative_speed());
                self.virtual_time.bump_clock_domain_epoch();
            }
            if fixed_clock_changed {
                self.fixed_time.set_timestep(applied.fixed_timestep());
                self.fixed_time.bump_clock_domain_epoch();
                self.reset_fixed_interpolation_history();
            }
            self.policy_generation = self.policy_generation.saturating_add(1);
        }
        Ok(WorldTimePolicyReceipt {
            previous,
            applied,
            generation: self.policy_generation,
            changed,
        })
    }

    /// 只设置暂停标志；旧增量和有效倍率缓存到下一次外帧推进才刷新。
    pub fn pause_virtual_time(&mut self) {
        self.virtual_time.pause();
    }

    /// 恢复虚拟采样并取消尚未消费的单步请求；不消除已存在的固定债务。
    pub fn unpause_virtual_time(&mut self) {
        self.single_step_requested = false;
        self.virtual_time.unpause();
    }

    /// 为下一次有效外帧预约一个暂停单步；要求已暂停、无活动步且没有重复预约。
    /// 成功只设置请求，不推进时钟；恢复或 World 替换会取消尚未消费的预约。
    pub fn request_single_step(&mut self) -> Result<(), WorldTimeControlError> {
        if !self.virtual_time.is_paused() {
            return Err(WorldTimeControlError::SingleStepRequiresPause);
        }
        if self.active_fixed_step.is_some() {
            return Err(WorldTimeControlError::FixedStepActive);
        }
        if self.single_step_requested {
            return Err(WorldTimeControlError::SingleStepAlreadyRequested);
        }
        self.single_step_requested = true;
        Ok(())
    }

    /// 接受 `outer` 后派生本 World 的虚拟增量、固定计划及只读快照。
    ///
    /// # 返回与错误
    ///
    /// 先拒绝重复、倒序和无凭据跳帧，故这些错误不消费单步请求或修改时钟。
    /// 返回计划尚未扣债，实际提交另经 begin/commit；来源版本随外帧复制到两个派生时钟。
    ///
    /// # 约束与生命周期
    ///
    /// 调用前须完成上一活动步的 commit/abort；该前提在此以 debug_assert 检查。
    /// 暂停通常计划零步；显式单步只补足一整步的欠债并消费预约，失败后的债务供重试保留。
    pub fn advance(
        &mut self,
        outer: FrameTimeSnapshot,
    ) -> Result<WorldTimeSnapshot, WorldTimeAdvanceError> {
        self.validate_outer_frame(outer)?;
        debug_assert!(
            self.active_fixed_step.is_none(),
            "a World fixed step must commit or abort before the next outer frame"
        );
        let source_generation = outer.real_clock_domain_stamp().source_generation();
        self.virtual_time
            .set_clock_domain_source_generation(source_generation);
        self.fixed_time
            .set_clock_domain_source_generation(source_generation);
        self.virtual_time
            .advance_from_real_delta(outer.raw_real_delta());
        let virtual_time_paused = self.virtual_time.is_paused();
        let single_step_requested = self.single_step_requested;
        if single_step_requested {
            // A failed single-step leaves its debt pending; retry only tops up to one step.
            let required_debt = self
                .fixed_time
                .timestep()
                .saturating_sub(self.fixed_time.overstep());
            self.fixed_time.accumulate_overstep(required_debt);
            self.single_step_requested = false;
        }
        let fixed_step_plan = if virtual_time_paused && !single_step_requested {
            self.fixed_time.plan_steps(0)
        } else if single_step_requested {
            // TODO: [CR-ZCA-R029-0001] 暂停且已预约单步时，外层预算为零仍计划一步；最大预算与调试特例的优先级尚未约定。
            // 下一步明确该优先级，并补暂停单步配合零外层预算的精确夹具；已有的非零预算用例未覆盖此组合。
            self.fixed_time.plan_steps(1)
        } else {
            self.fixed_time
                .accumulate_overstep(self.virtual_time.delta());
            self.fixed_time.plan_steps(outer.fixed_step_budget())
        };
        self.fixed_step_budget_remaining = fixed_step_plan.step_count;
        let snapshot = WorldTimeSnapshot::new(
            outer,
            self.virtual_time.delta(),
            self.virtual_time.elapsed(),
            virtual_time_paused,
            self.virtual_time.effective_speed_f64(),
            self.policy_generation,
            fixed_step_plan,
            self.virtual_time.clock_domain_stamp(),
            self.fixed_time.clock_domain_stamp(),
        );
        self.last_outer_frame_index = Some(outer.outer_frame_index());
        Ok(snapshot)
    }

    /// 用调用方取得的 `world_generation` 标记待提交步；本层不查询当前 World 身份。
    /// 已有活动步、无剩余名额或欠债不足时拒绝，成功只登记活动身份并返回非 Clone 令牌。
    /// 令牌的 elapsed 是成功提交后的预计端点，此时尚未扣债或增加固定步序号。
    pub(crate) fn begin_fixed_step(
        &mut self,
        world_generation: u64,
    ) -> Result<WorldFixedStep, WorldFixedStepError> {
        if let Some(active) = self.active_fixed_step {
            return Err(WorldFixedStepError::ActiveStep { active });
        }
        if self.fixed_step_budget_remaining == 0 {
            return Err(WorldFixedStepError::BudgetExhausted);
        }
        if self.fixed_time.overstep() < self.fixed_time.timestep() {
            return Err(WorldFixedStepError::InsufficientDebt);
        }
        let id = SimulationTickId::new(
            world_generation,
            self.fixed_time.clock_domain_stamp().epoch(),
            self.fixed_time.frame_index().saturating_add(1),
        );
        let step = WorldFixedStep::new(
            id,
            self.fixed_time.timestep(),
            self.fixed_time
                .elapsed()
                .saturating_add(self.fixed_time.timestep()),
        );
        self.active_fixed_step = Some(id);
        Ok(step)
    }

    /// 借用 `step` 并先匹配活动身份，再尝试扣除一整步；成功才推进时钟、扣名额并移动插值历史。
    /// 身份错误或债务不足保留原状态；World 代际检查由 Level 的外层提交入口完成。
    /// 返回成功不消费令牌对象，但重复提交会因不再有匹配活动步而被拒绝。
    pub(crate) fn commit_fixed_step(
        &mut self,
        step: &WorldFixedStep,
    ) -> Result<(), WorldFixedStepError> {
        self.validate_active_step(step)?;
        let previous_elapsed = self.fixed_time.elapsed();
        if !self.fixed_time.try_commit_step() {
            return Err(WorldFixedStepError::InsufficientDebt);
        }
        self.active_fixed_step = None;
        self.fixed_step_budget_remaining = self.fixed_step_budget_remaining.saturating_sub(1);
        self.previous_committed_fixed_step = self.current_committed_fixed_step;
        self.previous_committed_fixed_elapsed = previous_elapsed;
        self.current_committed_fixed_step = Some(step.id());
        Ok(())
    }

    /// 消费匹配的 `step` 并只清除活动身份；保留债务、名额、已提交时间及插值历史。
    /// 中止不回滚系统对 World 的写入；下一次 begin 可重用尚未提交的步序号。
    /// 参数按值传入；身份验证失败仍保留原活动身份，不等于已经中止。
    pub(crate) fn abort_fixed_step(
        &mut self,
        step: WorldFixedStep,
    ) -> Result<(), WorldFixedStepError> {
        self.validate_active_step(&step)?;
        self.active_fixed_step = None;
        Ok(())
    }

    /// 从已提交历史与当前实际债务复制插值证据；不使用外帧计划的预计余债或活动步。
    /// 历史为空时以当前已提交 elapsed 作为无 tick 基线，未必是 World 初建的零时刻。
    pub(crate) fn fixed_interpolation_context(&self) -> FixedInterpolationContext {
        let current_elapsed = self.fixed_time.elapsed();
        let remaining_debt = self.fixed_time.overstep();
        FixedInterpolationContext::new(
            FixedInterpolationState::new(
                self.previous_committed_fixed_step,
                self.previous_committed_fixed_elapsed,
            ),
            FixedInterpolationState::new(self.current_committed_fixed_step, current_elapsed),
            remaining_debt,
            self.fixed_time.timestep(),
            interpolation_fraction(remaining_debt, self.fixed_time.timestep()),
        )
    }

    /// 移除两个已提交步身份，并把前一端点重置为当前基线；保留时钟、债务、活动步及本帧名额。
    pub(crate) fn reset_fixed_interpolation_history(&mut self) {
        self.previous_committed_fixed_step = None;
        self.previous_committed_fixed_elapsed = self.fixed_time.elapsed();
        self.current_committed_fixed_step = None;
    }

    /// World 替换后清插值身份与未消费的单步预约，不重置累计时长或债务。
    /// 若替换发生在活动步中，外层 WorldDriver 检查代际并负责中止原令牌。
    pub(crate) fn reset_after_world_replacement(&mut self) {
        self.reset_fixed_interpolation_history();
        self.single_step_requested = false;
    }

    fn validate_active_step(&self, step: &WorldFixedStep) -> Result<(), WorldFixedStepError> {
        match self.active_fixed_step {
            Some(active) if active == step.id() => Ok(()),
            Some(active) => Err(WorldFixedStepError::ActiveStepMismatch {
                active,
                submitted: step.id(),
            }),
            None => Err(WorldFixedStepError::NoActiveStep {
                submitted: step.id(),
            }),
        }
    }

    // 初次接受任意外帧序号；后续只允许递增，跳过中间帧还须携带 discontinuity。
    // 凭据不豁免重复或倒序；本校验只读，不验证快照来自哪个 Runtime 实例。
    fn validate_outer_frame(&self, outer: FrameTimeSnapshot) -> Result<(), WorldTimeAdvanceError> {
        let submitted = outer.outer_frame_index();
        match self.last_outer_frame_index {
            Some(frame_index) if submitted == frame_index => {
                Err(WorldTimeAdvanceError::DuplicateOuterFrame { frame_index })
            }
            Some(last_consumed) if submitted < last_consumed => {
                Err(WorldTimeAdvanceError::OutOfOrderOuterFrame {
                    last_consumed,
                    submitted,
                })
            }
            Some(last_consumed)
                if last_consumed
                    .checked_add(1)
                    .is_some_and(|expected| submitted > expected)
                    && outer.discontinuity().is_none() =>
            {
                Err(WorldTimeAdvanceError::SkippedOuterFrames {
                    last_consumed,
                    submitted,
                })
            }
            _ => Ok(()),
        }
    }
}

/// Rejection returned before a World mutates time for an invalid outer-frame handoff.
/// 保存被拒绝的序号；调用方可保留旧 WorldTimeState 核对拒绝前后状态不变。
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum WorldTimeAdvanceError {
    /// 重复提交同一外帧；`frame_index` 为该已消费序号。
    #[error("outer frame {frame_index} was already consumed by this World")]
    DuplicateOuterFrame { frame_index: u64 },
    /// `submitted` 小于该 World 的 `last_consumed`。
    #[error(
        "outer frame {submitted} is older than this World's last consumed frame {last_consumed}"
    )]
    OutOfOrderOuterFrame { last_consumed: u64, submitted: u64 },
    /// 新序号跳过中间帧且未携带允许跳帧的外帧凭据。
    #[error(
        "outer frame {submitted} skipped one or more frames after {last_consumed} without a discontinuity"
    )]
    SkippedOuterFrames { last_consumed: u64, submitted: u64 },
}

/// 暂停单步预约的准入拒绝；各分支返回前不修改预约标志。
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum WorldTimeControlError {
    /// 当前暂停标志未设置；倍率为零本身不等于暂停。
    #[error("single-step requires paused virtual time")]
    SingleStepRequiresPause,
    /// 旧固定步仍待结算，不能再预约。
    #[error("cannot request single-step while a fixed step is active")]
    FixedStepActive,
    /// 尚有一个未由有效外帧消费的预约。
    #[error("a single-step request is already pending")]
    SingleStepAlreadyRequested,
}

// 对实际欠债先取整步余数再投影到 f32；零步长回退为零，接近一时可能舍入到 1.0。
fn interpolation_fraction(
    remaining_debt: std::time::Duration,
    timestep: std::time::Duration,
) -> f32 {
    if timestep.is_zero() {
        return 0.0;
    }
    let timestep_nanos = timestep.as_nanos();
    let remainder_nanos = remaining_debt.as_nanos() % timestep_nanos;
    (remainder_nanos as f64 / timestep_nanos as f64) as f32
}

/// Copyable observation of a World's mutable timing state outside its lock.
/// 锁内一次复制的观察值，不携带活动令牌或 World 身份；保留后不会随权威推进。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTimeState {
    /// 包含最近虚拟样本和当前暂停/倍率配置的独立时钟副本。
    virtual_time: Time<Virtual>,
    /// 已提交时间及实际未兑现债务的独立时钟副本。
    fixed_time: Time<Fixed>,
    /// 该控制器策略实际变化的饱和版本。
    policy_generation: u64,
    /// 该控制器最近成功接受的外帧序号，新建时为空。
    last_outer_frame_index: Option<u64>,
}

impl WorldTimeState {
    pub const fn virtual_time(self) -> Time<Virtual> {
        self.virtual_time
    }

    pub const fn fixed_time(self) -> Time<Fixed> {
        self.fixed_time
    }

    pub const fn policy_generation(self) -> u64 {
        self.policy_generation
    }

    pub const fn last_outer_frame_index(self) -> Option<u64> {
        self.last_outer_frame_index
    }
}

/// Immutable receipt for a World-local time-policy transaction.
/// 本 World 的策略提交结果，独立于 Runtime 默认策略回执；版本只在该控制器内解释。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTimePolicyReceipt {
    /// 本次 World 策略请求前的配置。
    previous: TimePolicy,
    /// 本次通过准入的请求配置；相等值不强制重写原位模式，不含暂停状态与欠债。
    applied: TimePolicy,
    /// 提交后的 World 策略版本，实际变化时饱和递增。
    generation: u64,
    /// 按值比较本次配置是否改变；无变化仍可返回成功。
    changed: bool,
}

impl WorldTimePolicyReceipt {
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

#[cfg(test)]
#[path = "tests/controller.rs"]
mod tests;

#[cfg(test)]
mod replacement_policy_tests {
    use super::*;

    #[test]
    fn replacement_epoch_exhaustion_rejects_without_mutating_clock_or_policy() {
        for fixed in [false, true] {
            let mut controller = WorldTimeController::default();
            if fixed {
                controller
                    .fixed_time
                    .try_rebase_clock_domain_epoch_after(u64::MAX - 1)
                    .unwrap();
            } else {
                controller
                    .virtual_time
                    .try_rebase_clock_domain_epoch_after(u64::MAX - 1)
                    .unwrap();
            }
            let before = controller.state();
            assert!(matches!(
                controller.prepare_world_replacement(
                    WorldReplacementTimeDisposition::Reset {
                        policy: TimePolicy::default(),
                        paused: false,
                    },
                    true,
                ),
                Err(WorldReplacementError::ClockEpochExhausted { .. })
            ));
            assert_eq!(controller.state(), before);
        }

        let mut controller = WorldTimeController::default();
        controller.policy_generation = u64::MAX;
        let before = controller.state();
        assert!(matches!(
            controller.prepare_world_replacement(
                WorldReplacementTimeDisposition::Reset {
                    policy: TimePolicy::default(),
                    paused: false,
                },
                true,
            ),
            Err(WorldReplacementError::PolicyGenerationExhausted { current: u64::MAX })
        ));
        assert_eq!(controller.state(), before);

        let mut clock = before.virtual_time();
        let unchanged = clock;
        assert!(clock
            .try_rebase_clock_domain_epoch_after(u64::MAX)
            .is_none());
        assert_eq!(clock, unchanged);
    }

    #[test]
    fn restore_rebases_checkpoint_epochs_without_rewinding_live_outer_source() {
        let mut live = WorldTimeController::default();
        live.virtual_time.set_clock_domain_source_generation(9);
        live.fixed_time.set_clock_domain_source_generation(9);
        live.last_outer_frame_index = Some(27);
        live.virtual_time
            .try_rebase_clock_domain_epoch_after(2)
            .unwrap();
        live.fixed_time
            .try_rebase_clock_domain_epoch_after(4)
            .unwrap();
        let mut checkpoint = live.state();
        checkpoint
            .virtual_time
            .set_clock_domain_source_generation(3);
        checkpoint.fixed_time.set_clock_domain_source_generation(3);
        checkpoint.last_outer_frame_index = Some(11);
        checkpoint
            .virtual_time
            .try_rebase_clock_domain_epoch_after(12)
            .unwrap();
        checkpoint
            .fixed_time
            .try_rebase_clock_domain_epoch_after(14)
            .unwrap();
        let before = live.state();
        let prepared = live
            .prepare_world_replacement(WorldReplacementTimeDisposition::Restore(checkpoint), true)
            .unwrap();
        assert_eq!(live.state(), before);
        let after = prepared.after();
        assert_eq!(after.last_outer_frame_index(), Some(27));
        assert_eq!(
            after
                .virtual_time()
                .clock_domain_stamp()
                .source_generation(),
            9
        );
        assert_eq!(
            after.fixed_time().clock_domain_stamp().source_generation(),
            9
        );
        assert_eq!(after.virtual_time().clock_domain_stamp().epoch(), 14);
        assert_eq!(after.fixed_time().clock_domain_stamp().epoch(), 16);
        assert_eq!(after.virtual_time().delta(), Duration::ZERO);
        assert_eq!(after.fixed_time().delta(), Duration::ZERO);
        live.commit_world_replacement(prepared);
        assert_eq!(live.state(), after);

        checkpoint
            .fixed_time
            .try_rebase_clock_domain_epoch_after(u64::MAX - 1)
            .unwrap();
        let before_rejection = live.state();
        assert!(matches!(
            live.prepare_world_replacement(
                WorldReplacementTimeDisposition::Restore(checkpoint),
                true,
            ),
            Err(WorldReplacementError::ClockEpochExhausted { .. })
        ));
        assert_eq!(live.state(), before_rejection);
    }
}
