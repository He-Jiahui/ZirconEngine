use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::framework::window::WindowRegistryId;

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
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::allocate_from;

    #[test]
    fn allocator_issues_the_final_nonzero_identity_once_then_reports_exhaustion() {
        let next_window_registry_id = AtomicU64::new(u64::MAX);

        assert_eq!(
            allocate_from(&next_window_registry_id).map(|identity| identity.raw()),
            Some(u64::MAX)
        );
        assert_eq!(allocate_from(&next_window_registry_id), None);
        assert_eq!(allocate_from(&next_window_registry_id), None);
    }
}
