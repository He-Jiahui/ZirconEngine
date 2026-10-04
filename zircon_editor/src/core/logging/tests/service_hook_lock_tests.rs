use std::sync::Arc;

use super::EditorLogService;
use crate::core::logging::{LogEntry, LogSeverity, LogSource};

#[test]
fn emission_hooks_release_their_configuration_locks_before_invocation() {
    let service = Arc::new(EditorLogService::default());
    let weak = Arc::downgrade(&service);
    let before_emission = Arc::new(move || {
        let service = weak.upgrade().expect("log service must outlive its hook");
        assert!(service.before_emission_hook.try_lock().is_ok());
    });
    let weak = Arc::downgrade(&service);
    let after_store = Arc::new(move |_: &crate::core::logging::LogRecord| {
        let service = weak.upgrade().expect("log service must outlive its hook");
        assert!(service.after_store_hook.try_lock().is_ok());
    });
    let weak = Arc::downgrade(&service);
    let before_event_dispatch = Arc::new(move || {
        let service = weak.upgrade().expect("log service must outlive its hook");
        assert!(service.before_event_dispatch_hook.try_lock().is_ok());
    });
    service.configure_emission_test_hooks(before_emission, after_store, before_event_dispatch);

    let record = service
        .store
        .push(LogEntry::new(LogSource::editor(), LogSeverity::Info, "hook", 0, None).unwrap())
        .unwrap();
    service.run_before_emission_hook();
    service.run_after_store_hook(&record);
    service.run_before_event_dispatch_hook();
}
