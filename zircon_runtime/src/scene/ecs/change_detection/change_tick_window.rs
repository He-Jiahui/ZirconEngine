use super::ChangeTick;

/// 一次查询或系统运行的变更观察窗口；SystemState 与 QueryState 共用它解释组件时钟。
/// all 用于独立查询，系统运行应传入上次运行时钟以免重复报告历史变化。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeTickWindow {
    last_run: ChangeTick,
    this_run: ChangeTick,
}

impl ChangeTickWindow {
    pub const fn new(last_run: ChangeTick, this_run: ChangeTick) -> Self {
        Self {
            last_run: last_run.clamp_older_than(this_run),
            this_run,
        }
    }

    pub const fn all(this_run: ChangeTick) -> Self {
        Self::new(ChangeTick::ZERO, this_run)
    }

    pub const fn last_run(self) -> ChangeTick {
        self.last_run
    }

    pub const fn this_run(self) -> ChangeTick {
        self.this_run
    }
}
