use super::*;

#[test]
fn headless_host_concurrent_reaper_never_waits_for_registry_lock() {
    let registry = OwnerRegistry::default();
    let _other_reaper = registry.0.lock().unwrap();
    assert!(!registry.finish_until(Instant::now()));
    assert!(registry.destroy_receipt().is_none());
    assert!(registry.begin().is_err());
}
