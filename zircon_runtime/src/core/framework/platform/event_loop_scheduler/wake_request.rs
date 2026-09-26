use std::time::Instant;

use super::{EventLoopClockDomain, EventLoopWakeSource};

/// 来源拥有待处理负载，这里只提交该来源当前的唤醒截止时间。
/// 同一来源再次提交会替换旧时间；消费方须另行从来源队列取出工作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventLoopWakeRequest {
    source: EventLoopWakeSource,
    domain: EventLoopClockDomain,
    deadline: Instant,
}

impl EventLoopWakeRequest {
    /// 请求下一次宿主轮询；调用方应传入与调度器相同单调时钟的当前时刻。
    pub const fn immediate(source: EventLoopWakeSource, now: Instant) -> Self {
        Self::at(source, EventLoopClockDomain::Monotonic, now)
    }

    pub const fn at(
        source: EventLoopWakeSource,
        domain: EventLoopClockDomain,
        deadline: Instant,
    ) -> Self {
        Self {
            source,
            domain,
            deadline,
        }
    }

    pub const fn source(self) -> EventLoopWakeSource {
        self.source
    }

    pub const fn domain(self) -> EventLoopClockDomain {
        self.domain
    }

    pub const fn deadline(self) -> Instant {
        self.deadline
    }
}
