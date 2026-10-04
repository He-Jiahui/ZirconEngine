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
