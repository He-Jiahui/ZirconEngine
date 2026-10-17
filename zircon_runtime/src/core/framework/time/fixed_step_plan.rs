use std::time::Duration;

/// 固定步欠债在一次观测时可执行的非破坏性计划。
///
/// # 计划与实际状态
///
/// 内建生产者按债务和本帧预算计算计划；`consumed` 与 `remaining_overstep`
/// 是假设计划中的所有步骤成功提交后的预测，不会提前扣除实际欠债。
/// World 的快照保留生成时的计划，之后提交或中止不会回写已拷贝的值；
/// 读取实际剩余债务或已提交插值状态时，应使用 World 当下的观测入口。
///
/// # 构造与生命周期
///
/// 字段公开，构造器不校验它们的相互关系；手工构造者负责保持语义一致。
/// 默认值的步长为零，各比例观察方法对此返回零；本值不是步提交凭证。
/// Result of draining fixed-timestep overstep for one outer update.
/// 外层帧对固定步欠债的非破坏性执行计划；实际扣债由 World 的逐步提交完成，失败可保留欠债重试。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixedStepPlan {
    /// 预算内计划执行的完整步数；生成或读取此值不代表任何一步已提交。
    pub step_count: u32,
    /// 解释此计划及其预计债务的步长；公开手工构造或默认值可为零。
    pub timestep: Duration,
    /// 计划中所有步骤成功时预计消费的时长；实际扣债由逐步提交完成。
    pub consumed: Duration,
    /// 所有计划步骤成功后的预计债务；中止后实际债务可能更高，应重新观察时钟。
    pub remaining_overstep: Duration,
}

impl FixedStepPlan {
    /// 组合一份固定步计划值，不操作时钟。
    ///
    /// # 参数
    ///
    /// - `step_count`：计划执行的整步数量。
    /// - `timestep`：解释每步和剩余债务所用的步长。
    /// - `consumed`：计划全部成功时预计消费的时长。
    /// - `remaining_overstep`：计划全部成功时预计保留的债务。
    ///
    /// # 返回与约束
    ///
    /// 返回可拷贝快照，不重算或校验参数间的关系。内建生产者用可用整步与预算计算这些值；
    /// 手工调用者负责一致性，构造成功不能证明任何一步已被提交。
    pub fn new(
        step_count: u32,
        timestep: Duration,
        consumed: Duration,
        remaining_overstep: Duration,
    ) -> Self {
        Self {
            step_count,
            timestep,
            consumed,
            remaining_overstep,
        }
    }

    /// Total unconsumed fixed-step debt after this outer frame.
    /// 读取计划全部成功后的预计余债副本；不回读 World 当前的实际欠债。
    pub fn debt_duration(&self) -> Duration {
        self.remaining_overstep
    }

    /// 按计划的步长观察预计债务中仍欠的完整步数。
    ///
    /// 返回向下取整的 `u128` 数量，不受本帧 `step_count` 预算截断，可能大于本帧计划数。
    /// 手工构造或默认值的步长为零时返回零；本观察不触发补步或提交。
    /// Number of complete fixed timesteps still owed after this outer frame.
    pub fn debt_whole_steps(&self) -> u128 {
        if self.timestep.is_zero() {
            return 0;
        }
        self.remaining_overstep.as_nanos() / self.timestep.as_nanos()
    }

    /// 将预计总债务按步长表示为调度或健康遥测的比例。
    ///
    /// 比例包括完整欠债及小数部分，可大于一，不能直接当作相邻状态的插值权重。
    /// 零步长返回 `0.0`；浮点结果是观察值，不用于扣债或确定精确的整步数量。
    /// Unbounded debt measured in fixed timesteps for scheduling and health telemetry.
    pub fn debt_timestep_ratio_f64(&self) -> f64 {
        if self.timestep.is_zero() {
            return 0.0;
        }
        self.remaining_overstep.as_secs_f64() / self.timestep.as_secs_f64()
    }

    /// 观察预计债务不足一整步的余数比例。
    ///
    /// 完整欠债先取余，不会挤进本权重；完整步数应另读 [debt_whole_steps](Self::debt_whole_steps)。
    /// 零步长返回 `0.0`。结果转为 `f32`，接近一的比例可能舍入到 `1.0`，
    /// 因此调用者不能据此要求严格小于一；读取不改变计划或提交状态。
    /// Fractional remainder of debt that can interpolate between two adjacent fixed states.
    pub fn interpolation_fraction(&self) -> f32 {
        if self.timestep.is_zero() {
            return 0.0;
        }
        let timestep_nanos = self.timestep.as_nanos();
        let remainder_nanos = self.remaining_overstep.as_nanos() % timestep_nanos;
        (remainder_nanos as f64 / timestep_nanos as f64) as f32
    }
}
