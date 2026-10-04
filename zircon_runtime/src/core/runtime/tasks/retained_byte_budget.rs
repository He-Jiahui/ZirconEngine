//! Cloneable RAII reservations for memory retained beyond task completion.

use std::sync::{Arc, Mutex, MutexGuard, Weak};

use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RetainedByteBudgetDiagnostics {
    pub capacity_bytes: usize,
    pub capacity_leases: usize,
    pub retained_bytes: usize,
    pub active_leases: usize,
    pub closed: bool,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RetainedByteBudgetError {
    #[error("retained byte budget is closed")]
    Closed,
    #[error(
        "retained byte budget has {remaining_bytes} bytes remaining; requested {requested_bytes}"
    )]
    CapacityExceeded {
        requested_bytes: usize,
        remaining_bytes: usize,
    },
    #[error("retained byte budget reached its {maximum_leases}-lease capacity")]
    LeaseCapacityExceeded { maximum_leases: usize },
    #[error("retained byte reservation overflowed the platform byte range")]
    Overflow,
}

/// 跨异步阶段保留结果的共享字节与租约预算；调用方须在持有结果期间保留租约。
/// close 只拒绝新租约，已有结果可继续使用直至最后一个租约克隆释放。
#[derive(Clone)]
pub struct RetainedByteBudget {
    inner: Arc<RetainedByteBudgetInner>,
}

struct RetainedByteBudgetInner {
    capacity_bytes: usize,
    capacity_leases: usize,
    state: Mutex<RetainedByteBudgetState>,
}

#[derive(Default)]
struct RetainedByteBudgetState {
    retained_bytes: usize,
    active_leases: usize,
    closed: bool,
}

/// A byte reservation released when its last clone is dropped.
#[derive(Clone)]
pub struct RetainedByteLease {
    inner: Arc<RetainedByteLeaseInner>,
}

struct RetainedByteLeaseInner {
    budget: Weak<RetainedByteBudgetInner>,
    retained_bytes: usize,
}

impl RetainedByteBudget {
    pub fn new(capacity_bytes: usize) -> Self {
        Self::with_max_leases(capacity_bytes, usize::MAX)
    }

    pub fn with_max_leases(capacity_bytes: usize, capacity_leases: usize) -> Self {
        Self {
            inner: Arc::new(RetainedByteBudgetInner {
                capacity_bytes,
                capacity_leases,
                state: Mutex::new(RetainedByteBudgetState::default()),
            }),
        }
    }

    /// 在同一锁下同时预留字节和租约名额；失败时不会接管调用方的结果。
    pub fn try_reserve(
        &self,
        retained_bytes: usize,
    ) -> Result<RetainedByteLease, RetainedByteBudgetError> {
        let mut state = lock(&self.inner.state);
        if state.closed {
            return Err(RetainedByteBudgetError::Closed);
        }
        if state.active_leases >= self.inner.capacity_leases {
            return Err(RetainedByteBudgetError::LeaseCapacityExceeded {
                maximum_leases: self.inner.capacity_leases,
            });
        }
        let next_retained_bytes = state
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or(RetainedByteBudgetError::Overflow)?;
        if next_retained_bytes > self.inner.capacity_bytes {
            return Err(RetainedByteBudgetError::CapacityExceeded {
                requested_bytes: retained_bytes,
                remaining_bytes: self
                    .inner
                    .capacity_bytes
                    .saturating_sub(state.retained_bytes),
            });
        }
        state.retained_bytes = next_retained_bytes;
        state.active_leases = state.active_leases.saturating_add(1);
        drop(state);
        Ok(RetainedByteLease {
            inner: Arc::new(RetainedByteLeaseInner {
                budget: Arc::downgrade(&self.inner),
                retained_bytes,
            }),
        })
    }

    pub fn diagnostics(&self) -> RetainedByteBudgetDiagnostics {
        let state = lock(&self.inner.state);
        RetainedByteBudgetDiagnostics {
            capacity_bytes: self.inner.capacity_bytes,
            capacity_leases: self.inner.capacity_leases,
            retained_bytes: state.retained_bytes,
            active_leases: state.active_leases,
            closed: state.closed,
        }
    }

    pub fn close(&self) {
        lock(&self.inner.state).closed = true;
    }
}

impl RetainedByteLease {
    pub fn retained_bytes(&self) -> usize {
        self.inner.retained_bytes
    }
}

// 共享 lease 仅在最后一个 Arc 所有者释放时归还字节与名额；预算已销毁时 Weak 升级失败便结束。
impl Drop for RetainedByteLeaseInner {
    fn drop(&mut self) {
        let Some(budget) = self.budget.upgrade() else {
            return;
        };
        let mut state = lock(&budget.state);
        state.retained_bytes = state.retained_bytes.saturating_sub(self.retained_bytes);
        state.active_leases = state.active_leases.saturating_sub(1);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
#[path = "retained_byte_budget/tests/cases.rs"]
mod tests;
