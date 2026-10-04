use std::sync::Arc;

use super::EditorI18nService;

#[test]
fn locale_hooks_release_their_configuration_locks_before_invocation() {
    let service = Arc::new(EditorI18nService::default());
    let weak = Arc::downgrade(&service);
    service.configure_before_event_dispatch_hook(Arc::new(move || {
        let service = weak.upgrade().expect("i18n service must outlive its hook");
        assert!(service.before_event_dispatch_hook.try_lock().is_ok());
    }));
    service.run_before_event_dispatch_hook();

    let weak = Arc::downgrade(&service);
    service.configure_after_failure_locale_read_hook(Arc::new(move || {
        let service = weak.upgrade().expect("i18n service must outlive its hook");
        assert!(service.after_failure_locale_read_hook.try_lock().is_ok());
    }));
    service.run_after_failure_locale_read_hook();

    let weak = Arc::downgrade(&service);
    service.configure_after_locale_capture_hook(Arc::new(move || {
        let service = weak.upgrade().expect("i18n service must outlive its hook");
        assert!(service.after_locale_capture_hook.try_lock().is_ok());
    }));
    service.run_after_locale_capture_hook();
}
