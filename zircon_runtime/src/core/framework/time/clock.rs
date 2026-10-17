use std::marker::PhantomData;
use std::time::Duration;

use super::{ClockDomainMarker, ClockDomainStamp, Fixed, FixedStepPlan, MonotonicReal, Virtual};

/// Read-only clock observation shared by the engine-owned clock domains.
/// 按时钟域区分的只读观测值；外层 Runtime 推进真实时间，World 再派生虚拟时间与固定步，调用方不可混用其 delta。
///
/// # 所有权与生命周期
///
/// 内建上下文下，此值可独立复制；副本不会重新采样，也不跟随所有者的后续推进。
/// [context](Self::context) 只借用本值中的上下文；读取内建上下文不会取得 Runtime 或 World 的推进权。
///
/// # 参考
///
/// 真实采样由 [FrameClock](crate::core::FrameClock) 完成；World 以同一外帧输入派生自己的两个时钟。
/// 域类别与代际由 [ClockDomainStamp] 描述，不能单独充当 World 实例身份。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Time<T: ClockDomainMarker = MonotonicReal> {
    /// 随本观测值持有的域配置或标记。
    context: T,
    /// 最近一次本域推进的增量；新值尚未推进时为零。
    delta: Duration,
    /// 本值自构造起饱和累加的本域时长。
    elapsed: Duration,
    /// 本域的推进序号；固定域在成功提交固定步后增加。
    frame_index: u64,
    /// 随观测保存的域类别、配置 epoch 和采样来源代际。
    clock_domain_stamp: ClockDomainStamp,
    marker: PhantomData<T>,
}

impl<T: ClockDomainMarker + Default> Default for Time<T> {
    fn default() -> Self {
        Self::new_with(T::default())
    }
}

impl<T: ClockDomainMarker> Time<T> {
    /// 使用 `context` 构造尚未推进的本地时钟值。
    ///
    /// # 返回与约束
    ///
    /// 增量、累计时长和序号均为零；初始 stamp 从上下文的域类别取得，两个代际均为零。
    /// 不采样、不注册时钟，也不校验自定义上下文与输入时间的来源关系。
    pub fn new_with(context: T) -> Self {
        Self {
            context,
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
            frame_index: 0,
            clock_domain_stamp: ClockDomainStamp::initial(T::CLOCK_DOMAIN),
            marker: PhantomData,
        }
    }

    /// 借用本值持有的上下文；引用的有效期受本次 `self` 借用限制。
    pub fn context(&self) -> &T {
        &self.context
    }

    // 仅借用本值的上下文；调用方仍负责配置变化所需的状态检查与代际更新。
    pub(crate) fn context_mut(&mut self) -> &mut T {
        &mut self.context
    }

    /// 读取最近一次本域推进的增量副本；固定域表示单步增量，不能据此读取本外帧所有步的累计量。
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// 将 [delta](Self::delta) 转为浮点秒；转换可舍入，不用于精确扣除固定欠债。
    pub fn delta_secs_f64(&self) -> f64 {
        self.delta.as_secs_f64()
    }

    /// 读取本域自构造起累计的时长副本；固定域只累加已提交步的时长。
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// 将 [elapsed](Self::elapsed) 转为可舍入的浮点秒，不改变原 Duration 值。
    pub fn elapsed_secs_f64(&self) -> f64 {
        self.elapsed.as_secs_f64()
    }

    /// 读取本域推进序号；真实与虚拟域按各自推进增加，固定域按提交步数增加。
    /// 零增量推进也增加序号，达到 `u64::MAX` 后保持饱和。
    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    /// 复制当前域的 stamp；它描述类别与代际，不认证观测来自哪个 World。
    pub fn clock_domain_stamp(&self) -> ClockDomainStamp {
        self.clock_domain_stamp
    }

    // 由所有者在接受影响本域的配置变化后调用；计数饱和递增，不重置累计时间。
    pub(crate) fn bump_clock_domain_epoch(&mut self) {
        self.clock_domain_stamp.bump_epoch();
    }

    // Prepare a fresh timeline identity without a dependency on a scene owner.
    // Failure leaves this value unchanged; the caller decides how to report it.
    pub(crate) fn try_rebase_clock_domain_epoch_after(&mut self, floor: u64) -> Option<()> {
        let stamp = self.clock_domain_stamp.checked_epoch_after(floor)?;
        self.clock_domain_stamp = stamp;
        self.delta = Duration::ZERO;
        Some(())
    }

    // 保存上游采样来源代际；此处直接赋值，单调性由传入凭据的所有者维护。
    pub(crate) fn set_clock_domain_source_generation(&mut self, source_generation: u64) {
        self.clock_domain_stamp
            .set_source_generation(source_generation);
    }

    // 时钟权威推进后才发布新的帧索引；单独读旧快照不能代表下一帧状态。
    // 即使 delta 为零也发布一次推进；累计时长和序号均饱和增加，本层不采样或验证来源。
    pub(crate) fn advance_by(&mut self, delta: Duration) {
        self.delta = delta;
        self.elapsed = self.elapsed.saturating_add(delta);
        self.frame_index = self.frame_index.saturating_add(1);
    }
}

impl Time<Virtual> {
    /// 消费外帧的 `real_delta`，先采样暂停/倍率，再缩放、截断并发布一次虚拟推进。
    /// 零真实增量或暂停仍刷新有效倍率和推进序号；累计虚拟时长只增加缩放后的量。
    pub(crate) fn advance_from_real_delta(&mut self, real_delta: Duration) {
        let effective_speed = self.context.effective_speed_for_next_delta();
        let scaled = scale_virtual_delta(real_delta, effective_speed, self.context.max_delta());
        self.advance_by(scaled);
    }

    /// 读取变速后单次虚拟增量的上限，返回 Duration 副本。
    pub fn max_delta(&self) -> Duration {
        self.context.max_delta()
    }

    /// 设置非零虚拟增量上限；零值会 panic，策略准入和 epoch 更新由 World 完成。
    pub(crate) fn set_max_delta(&mut self, max_delta: Duration) {
        self.context.set_max_delta(max_delta);
    }

    /// 读取独立暂停标志；零倍率也可产生零增量，但不等于暂停。
    pub fn is_paused(&self) -> bool {
        self.context.is_paused()
    }

    /// 置暂停标志；有效倍率缓存要到下一次虚拟推进才刷新。
    pub(crate) fn pause(&mut self) {
        self.context.pause();
    }

    /// 清除暂停标志；不修改已有累计时间，也不立即刷新有效倍率缓存。
    pub(crate) fn unpause(&mut self) {
        self.context.unpause();
    }

    /// 读取配置倍率；暂停时仍保留配置值，不代表最近一次推进实际使用的倍率。
    pub fn relative_speed_f64(&self) -> f64 {
        self.context.relative_speed_f64()
    }

    /// 保存有限、非负倍率（允许负零）；非法值会 panic，不立即刷新有效倍率缓存。
    /// World 应先完成策略准入，接受变化后另行更新本域 epoch。
    pub(crate) fn set_relative_speed_f64(&mut self, speed: f64) {
        self.context.set_relative_speed_f64(speed);
    }

    /// 读取最近一次虚拟推进采样的倍率，初始值为 `1.0`。
    /// 暂停或配置变更可以先发生；本值到下一次推进才随之刷新。
    pub fn effective_speed_f64(&self) -> f64 {
        self.context.effective_speed_f64()
    }
}

// 调用方提供 Virtual 已约束的有限非负倍率与非零上限；真实增量或倍率为零时先返回零。
// 乘积非有限或达到上限时直接截断；转为 Duration 失败也回退到上限。
fn scale_virtual_delta(
    real_delta: Duration,
    effective_speed: f64,
    max_delta: Duration,
) -> Duration {
    if real_delta.is_zero() || effective_speed == 0.0 {
        return Duration::ZERO;
    }

    // Apply the clamp before constructing a Duration: `Duration::mul_f64` can
    // panic for a valid finite speed whose scaled duration exceeds its range.
    let scaled_seconds = real_delta.as_secs_f64() * effective_speed;
    if !scaled_seconds.is_finite() || scaled_seconds >= max_delta.as_secs_f64() {
        return max_delta;
    }

    Duration::try_from_secs_f64(scaled_seconds)
        .map(|scaled| scaled.min(max_delta))
        .unwrap_or(max_delta)
}

impl Time<Fixed> {
    /// 使用 `timestep` 新建零欠债、尚未提交固定步的时钟值。
    ///
    /// # 约束
    ///
    /// 步长必须非零，否则 panic；本构造不创建 World 的活动步或预算。
    pub fn from_duration(timestep: Duration) -> Self {
        let mut time = Self::default();
        time.set_timestep(timestep);
        time
    }

    /// 读取单个固定步提交时使用的非零持续时间。
    pub fn timestep(&self) -> Duration {
        self.context.timestep()
    }

    /// 设置非零步长并保留欠债；零值会 panic，不自行检查活动步或旧债务。
    /// 新建时使用零欠债；World 变更须先准入，接受后另行更新 epoch 与插值历史。
    pub(crate) fn set_timestep(&mut self, timestep: Duration) {
        self.context.set_timestep(timestep);
    }

    /// 复制尚未兑现的全部固定欠债，包括整步与余数；读取不会提交或扣债。
    pub fn overstep(&self) -> Duration {
        self.context.overstep()
    }

    /// 将本域的 `delta` 饱和加入欠债；不推进已提交时长或序号。
    pub(crate) fn accumulate_overstep(&mut self, delta: Duration) {
        self.context.accumulate_overstep(delta);
    }

    /// Describes debt that can be consumed this frame without mutating the clock.
    /// 按 `max_steps` 截断可用整步，返回全部计划成功后的消费量和预计余债。
    /// 查询不扣债；计划不保存活动步身份，不能充当 World 的提交凭证。
    pub(crate) fn plan_steps(&self, max_steps: u32) -> FixedStepPlan {
        let timestep = self.timestep();
        let step_count = self.context.available_steps(max_steps);
        let consumed = timestep.saturating_mul(step_count);
        FixedStepPlan::new(
            step_count,
            timestep,
            consumed,
            self.overstep().saturating_sub(consumed),
        )
    }

    /// Commits one already-begun fixed step if sufficient debt remains.
    /// 债务不足返回 `false` 并保留观测；成功扣一整步、推进单步时长并返回 `true`。
    /// 本层只检查欠债；World 调用方须先验证活动步令牌与所属 World 的代际。
    pub(crate) fn try_commit_step(&mut self) -> bool {
        if self.context.take_steps(1) == 0 {
            return false;
        }
        let timestep = self.timestep();
        self.advance_by(timestep);
        true
    }

    /// 测试专用的批量扣债与推进；返回原计划，零可用步时保留旧 delta、时长和序号。
    /// 生产路径由 World 逐步提交，此辅助入口不执行活动步或 World 身份准入。
    #[cfg(test)]
    pub(crate) fn drain_steps(&mut self, max_steps: u32) -> FixedStepPlan {
        let plan = self.plan_steps(max_steps);
        let step_count = self.context.take_steps(max_steps);
        if step_count > 0 {
            self.delta = plan.timestep;
            self.elapsed = self.elapsed.saturating_add(plan.consumed);
            self.frame_index = self.frame_index.saturating_add(u64::from(step_count));
        }
        plan
    }
}
