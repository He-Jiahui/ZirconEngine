use std::time::Instant;

use super::EventLoopClockDomain;

/// 交给宿主事件循环适配器的下一轮等待策略；截止时间只在单调时钟域内比较。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopControlFlow {
    Poll,
    Wait,
    WaitUntil {
        domain: EventLoopClockDomain,
        deadline: Instant,
    },
}
