use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::framework::window::WindowRegistryId;

// 0 标记身份永久耗尽；最后一个非零值发出后不再分配，避免新 driver 复用旧 registry 身份。
static NEXT_WINDOW_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

/// Allocates one process-unique platform-host identity without reusing a
/// value after a driver has been torn down. Driver construction is cold, so
/// relaxed atomic ordering is sufficient: uniqueness is the only shared fact.
pub(in crate::platform) fn allocate_window_registry_id() -> Option<WindowRegistryId> {
    allocate_from(&NEXT_WINDOW_REGISTRY_ID)
}

fn allocate_from(next_window_registry_id: &AtomicU64) -> Option<WindowRegistryId> {
    let raw = next_window_registry_id
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            if current == 0 {
                return None;
            }
            Some(match current.checked_add(1) {
                Some(next) => next,
                None => 0,
            })
        })
        .ok()?;
    WindowRegistryId::new(raw)
}

#[cfg(test)]
#[path = "tests/registry_id_allocator.rs"]
mod tests;
