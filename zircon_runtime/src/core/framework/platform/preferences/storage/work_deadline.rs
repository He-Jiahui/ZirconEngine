use std::time::Instant;

/// 工作开始的截止时间；它不限制后端执行时长，也不同于票据等待的观察截止时间。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreferenceWorkDeadline {
    deadline: Option<Instant>,
}

impl PreferenceWorkDeadline {
    pub const fn none() -> Self {
        Self { deadline: None }
    }

    pub const fn at(deadline: Instant) -> Self {
        Self {
            deadline: Some(deadline),
        }
    }

    pub const fn instant(self) -> Option<Instant> {
        self.deadline
    }
}
