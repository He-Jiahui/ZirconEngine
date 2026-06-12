use super::super::super::super::*;
use super::support::cpal_backend;

#[test]
fn cpal_backend_reports_realtime_capabilities() {
    let sound = DefaultSoundManager::default();
    let backend = cpal_backend(&sound);

    assert!(backend.realtime_capable);
    assert!(!backend.deterministic);
}
