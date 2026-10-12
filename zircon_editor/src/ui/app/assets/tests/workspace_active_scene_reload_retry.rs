use std::time::Duration;

use super::next_active_scene_reload_admission_retry;

#[test]
fn admission_retry_backs_off_three_times_then_terminates() {
    assert_eq!(
        next_active_scene_reload_admission_retry(None),
        Some((1, Duration::from_millis(64)))
    );
    assert_eq!(
        next_active_scene_reload_admission_retry(Some(1)),
        Some((2, Duration::from_millis(128)))
    );
    assert_eq!(
        next_active_scene_reload_admission_retry(Some(2)),
        Some((3, Duration::from_millis(256)))
    );
    assert_eq!(next_active_scene_reload_admission_retry(Some(3)), None);
}
