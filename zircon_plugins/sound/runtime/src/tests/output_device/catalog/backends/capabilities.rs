use super::super::super::super::*;
use super::support::software_null_backend;

#[test]
fn software_null_backend_reports_deterministic_capabilities() {
    let sound = DefaultSoundManager::default();
    let backend = software_null_backend(&sound);

    assert!(backend.deterministic);
    assert!(!backend.realtime_capable);
    assert!(backend.max_sample_rate_hz >= 48_000);
    assert!(backend.max_channel_count >= 2);
}
