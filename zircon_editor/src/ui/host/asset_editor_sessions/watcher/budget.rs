use std::fmt;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiAssetWatchBudget {
    pub(super) max_pending_paths: usize,
    pub(super) max_paths_per_poll: usize,
    pub(super) max_poll_time: Duration,
}

impl UiAssetWatchBudget {
    pub(super) fn try_new(
        max_pending_paths: usize,
        max_paths_per_poll: usize,
        max_poll_time: Duration,
    ) -> Result<Self, UiAssetWatchBudgetError> {
        if max_pending_paths == 0 {
            return Err(UiAssetWatchBudgetError::ZeroPendingPathCapacity);
        }
        if max_paths_per_poll == 0 {
            return Err(UiAssetWatchBudgetError::ZeroPathsPerPoll);
        }
        if max_poll_time.is_zero() {
            return Err(UiAssetWatchBudgetError::ZeroPollTime);
        }
        Ok(Self {
            max_pending_paths,
            max_paths_per_poll,
            max_poll_time,
        })
    }

    pub(super) fn start_poll(self) -> UiAssetWatchPollAllowance {
        UiAssetWatchPollAllowance {
            remaining_items: self.max_paths_per_poll,
            deadline: Instant::now() + self.max_poll_time,
            consumed_items: 0,
        }
    }
}

#[derive(Debug)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetWatchPollAllowance {
    remaining_items: usize,
    deadline: Instant,
    consumed_items: usize,
}

impl UiAssetWatchPollAllowance {
    pub(in crate::ui::host::asset_editor_sessions) fn try_take(&mut self) -> bool {
        if self.remaining_items == 0 || Instant::now() >= self.deadline {
            return false;
        }
        self.remaining_items -= 1;
        self.consumed_items += 1;
        true
    }

    pub(in crate::ui::host::asset_editor_sessions) fn remaining_items(&self) -> usize {
        self.remaining_items
    }

    pub(in crate::ui::host::asset_editor_sessions) fn exhausted(&self) -> bool {
        self.remaining_items == 0 || Instant::now() >= self.deadline
    }

    #[cfg(test)]
    pub(super) fn expire_for_test(&mut self) {
        self.deadline = Instant::now();
    }

    #[cfg(test)]
    pub(super) fn consumed_items_for_test(&self) -> usize {
        self.consumed_items
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiAssetWatchBudgetError {
    ZeroPendingPathCapacity,
    ZeroPathsPerPoll,
    ZeroPollTime,
}

impl fmt::Display for UiAssetWatchBudgetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ZeroPendingPathCapacity => {
                "UI asset watch pending-path capacity must be non-zero"
            }
            Self::ZeroPathsPerPoll => "UI asset watch per-poll path budget must be non-zero",
            Self::ZeroPollTime => "UI asset watch per-poll time budget must be non-zero",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for UiAssetWatchBudgetError {}
