use std::sync::Arc;

use super::PlayPendingEditDecisionAdapter;

#[test]
fn publish_hook_runs_after_releasing_configuration_lock() {
    let adapter = Arc::new(PlayPendingEditDecisionAdapter::default());
    let weak = Arc::downgrade(&adapter);
    adapter.configure_before_publish_state_lock_hook(Arc::new(move || {
        let adapter = weak.upgrade().expect("adapter must outlive its hook");
        assert!(
            adapter.before_publish_state_lock_hook.try_lock().is_ok(),
            "a concurrent publisher must be able to enter the hook"
        );
    }));

    adapter.run_before_publish_state_lock_hook();
}
