use std::panic::{self, AssertUnwindSafe};

use crate::core::CoreRuntime;

#[test]
fn core_handle_registry_accessors_recover_poisoned_runtime_locks() {
    let runtime = CoreRuntime::new();
    let handle = runtime.handle();

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.inner.modules.lock().unwrap();
        panic!("poison core handle modules registry");
    }));
    assert!(handle.lock_modules().is_empty());

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.inner.services.lock().unwrap();
        panic!("poison core handle services registry");
    }));
    assert!(handle.lock_services().is_empty());

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.inner.frozen_module_graph.lock().unwrap();
        panic!("poison core handle frozen module graph");
    }));
    assert!(handle.lock_frozen_module_graph().is_none());

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.inner.active_module_order.lock().unwrap();
        panic!("poison core handle active module order");
    }));
    assert!(handle.lock_active_module_order().is_empty());

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.inner.lifecycle_coordinator.lock().unwrap();
        panic!("poison core handle lifecycle coordinator");
    }));
    drop(handle.lock_lifecycle_coordinator());

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.inner.devtools_plugin_catalog_entries.lock().unwrap();
        panic!("poison core handle devtools plugin catalog entries");
    }));
    handle.replace_devtools_plugin_catalog_entries(Vec::new());

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle
            .inner
            .runtime_module_lifecycle_observer
            .lock()
            .unwrap();
        panic!("poison core handle runtime module lifecycle observer");
    }));
    assert!(handle.lock_runtime_module_lifecycle_observer().is_none());
}
