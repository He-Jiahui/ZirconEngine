use std::time::Duration;

/// 固定步时钟上下文中的步长与尚未兑现的虚拟时间。
///
/// # 不变量与推进责任
///
/// 默认步长和内部设置入口保证 `timestep` 非零，供整步数量计算使用。
/// `overstep` 包含完整欠债及小数余量；帧推进先积累，生产路径在每个固定步提交时扣债。
/// 此上下文本身不保存 elapsed 或帧索引，扣债后的时钟推进由外层 [Time](super::Time) 完成。
///
/// # 观察边界
///
/// 读取债务或生成 [FixedStepPlan](super::FixedStepPlan) 不代表步已执行。
/// World 的中止路径保留欠债用于重试，实际步长改变还须经过 World 的状态校验。
/// Fixed-step clock marker and accumulator state.
/// World 固定步的欠债累加器；先按外层帧积累虚拟时间，成功提交的步才扣除欠债。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fixed {
    timestep: Duration,
    overstep: Duration,
}

impl Default for Fixed {
    fn default() -> Self {
        Self {
            timestep: Duration::from_micros(15_625),
            overstep: Duration::ZERO,
        }
    }
}

impl Fixed {
    pub fn timestep(&self) -> Duration {
        self.timestep
    }

    // 这里仅防止零步长，并保留已有欠债；它不自行检查活动步或债务状态。
    // Time 的新建入口使用零债务，World 的变更入口先完成状态检查，才调用本设置入口。
    pub(crate) fn set_timestep(&mut self, timestep: Duration) {
        assert_ne!(timestep, Duration::ZERO, "fixed timestep must be non-zero");
        self.timestep = timestep;
    }

    /// 观察尚未兑现的全部虚拟时间债务。
    ///
    /// 返回 Duration 副本，包含超过当前预算的整步和不足一整步的余数。
    /// 该值尚未推进固定时钟，不能当作已提交的 elapsed；读取不消费欠债。
    pub fn overstep(&self) -> Duration {
        self.overstep
    }

    pub(crate) fn accumulate_overstep(&mut self, delta: Duration) {
        self.overstep = self.overstep.saturating_add(delta);
    }

    pub(crate) fn available_steps(&self, max_steps: u32) -> u32 {
        (self.overstep.as_nanos() / self.timestep.as_nanos()).min(u128::from(max_steps)) as u32
    }

    // 生产路径逐步提交时只取一整步，测试 drain 可一次取多步；本入口只扣债。
    // 返回实际消费数；非零时，调用方仍须推进 Time 的 elapsed 与帧索引，不能把计划查询当作这个提交边界。
    pub(crate) fn take_steps(&mut self, max_steps: u32) -> u32 {
        let available_steps = self.available_steps(max_steps);
        let consumed = self.timestep.saturating_mul(available_steps);
        self.overstep = self.overstep.saturating_sub(consumed);
        available_steps
    }
}
