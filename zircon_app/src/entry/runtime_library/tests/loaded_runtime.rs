use std::sync::atomic::{AtomicU64, Ordering};

use super::{allocate_runtime_instance_id, LoadedRuntime};
use crate::entry::runtime_library::runtime_library_environment_override_request;

#[test]
fn environment_override_load_failure_keeps_the_override_request_provenance() {
    let path = std::env::temp_dir().join(format!(
        "zircon_missing_runtime_override_{}_{}.dll",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos()
    ));
    let request = runtime_library_environment_override_request(&path);
    let error = match LoadedRuntime::load_for_request(&path, request.clone()) {
        Ok(_) => panic!("a nonexistent environment override must fail to load"),
        Err(error) => error,
    };
    let diagnostic = error.to_string();

    assert!(diagnostic.contains(&format!("requested_path={request}")));
    assert!(diagnostic.contains("cause="));
    assert!(diagnostic.contains("recovery=stage the runtime library"));
}

#[test]
fn runtime_instance_allocator_returns_distinct_nonzero_ids() {
    let counter = AtomicU64::new(1);
    assert_eq!(allocate_runtime_instance_id(&counter).unwrap(), 1);
    assert_eq!(allocate_runtime_instance_id(&counter).unwrap(), 2);
    assert_eq!(counter.load(Ordering::Relaxed), 3);
}

#[test]
fn runtime_instance_allocator_exhaustion_never_wraps_or_reuses_an_id() {
    let counter = AtomicU64::new(u64::MAX);
    assert_eq!(allocate_runtime_instance_id(&counter).unwrap(), u64::MAX);
    assert_eq!(counter.load(Ordering::Relaxed), 0);
    for _ in 0..2 {
        assert!(allocate_runtime_instance_id(&counter)
            .unwrap_err()
            .to_string()
            .contains("identity space exhausted"));
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }
}
