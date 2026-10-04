//! 异步基础管线未就绪时的事件循环检查时钟。
//! 退避限制重复呈现；一次性任务还需以自身终止期限裁剪下一次唤醒。

use std::time::{Duration, Instant};

// Keeps the first pending recheck responsive while avoiding continuous full-frame presents.
pub(super) const BASE_PIPELINE_RECHECK_INTERVAL: Duration = Duration::from_millis(16);
pub(super) const BASE_PIPELINE_RECHECK_MAX_INTERVAL: Duration = Duration::from_millis(250);
pub(super) const ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT: Duration = Duration::from_secs(45);

/// 供事件循环设置下一次异步管线检查；一次性任务传入终止上界，交互任务可省略。
pub(super) fn base_pipeline_recheck_deadline_with_cap(
    now: Instant,
    retry_attempt: u32,
    deadline_cap: Option<Instant>,
) -> Instant {
    let deadline = now + base_pipeline_recheck_interval(retry_attempt);
    deadline_cap.map_or(deadline, |deadline_cap| deadline.min(deadline_cap))
}

pub(super) fn base_pipeline_recheck_is_due(deadline: Instant, now: Instant) -> bool {
    now >= deadline
}

pub(super) fn one_shot_base_pipeline_wait_deadline(started_at: Instant) -> Instant {
    started_at + ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT
}

pub(super) fn one_shot_base_pipeline_wait_is_expired(started_at: Instant, now: Instant) -> bool {
    now >= one_shot_base_pipeline_wait_deadline(started_at)
}

fn base_pipeline_recheck_interval(retry_attempt: u32) -> Duration {
    let multiplier = 1_u32 << retry_attempt.min(4);
    BASE_PIPELINE_RECHECK_INTERVAL
        .saturating_mul(multiplier)
        .min(BASE_PIPELINE_RECHECK_MAX_INTERVAL)
}

#[cfg(test)]
#[path = "tests/base_pipeline_recheck.rs"]
mod tests;
