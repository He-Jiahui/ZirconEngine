use crate::kira_bridge::KIRA_CPAL_BACKEND;

use super::support::kira_catalog_fixture;

#[test]
fn kira_cpal_is_the_only_runtime_output_backend() {
    let fixture = kira_catalog_fixture();

    assert_eq!(fixture.backends.len(), 1);
    let backend = &fixture.backends[0];
    assert_eq!(backend.backend, KIRA_CPAL_BACKEND);
    assert!(backend.realtime_capable);
    assert!(!backend.deterministic);
    assert_eq!(backend.max_channel_count, 2);
    assert_ne!(backend.backend, "software-null");
    assert_ne!(backend.backend, "software-test");
    assert_ne!(backend.backend, "software-preview");
}

#[test]
fn output_device_catalog_never_synthesizes_retired_software_devices() {
    let fixture = kira_catalog_fixture();

    assert!(fixture.devices.iter().all(|device| {
        device.descriptor.backend == KIRA_CPAL_BACKEND
            && !device.descriptor.id.as_str().starts_with("software-")
    }));
}
