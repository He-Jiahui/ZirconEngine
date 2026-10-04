use std::time::Duration;

use super::{AutosavePolicy, AutosaveScheduler};

#[test]
fn policy_update_recalculates_the_next_deadline_from_the_existing_anchor() {
    let mut scheduler = AutosaveScheduler::new(AutosavePolicy::default());

    scheduler.update_policy(AutosavePolicy::new(Duration::from_secs(60)).unwrap());

    assert!(scheduler.is_due(Duration::from_secs(100)));
}

#[test]
fn policy_update_does_not_make_a_longer_interval_due_early() {
    let mut scheduler =
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(60)).unwrap());

    scheduler.update_policy(AutosavePolicy::new(Duration::from_secs(300)).unwrap());

    assert!(!scheduler.is_due(Duration::from_secs(20)));
    assert!(scheduler.is_due(Duration::from_secs(300)));
}
