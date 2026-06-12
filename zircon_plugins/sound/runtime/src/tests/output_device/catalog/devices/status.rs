use super::super::super::super::*;
use super::support::software_null_picker;

#[test]
fn configured_software_null_picker_projects_output_device_status() {
    let sound = DefaultSoundManager::default();
    let descriptor = software_null_picker(&sound).descriptor;

    sound.configure_output_device(descriptor.clone()).unwrap();
    let status = sound.output_device_status().unwrap();
    assert_eq!(status.descriptor, descriptor);
    assert_eq!(status.latency.requested_latency_blocks, 2);
    assert_eq!(
        status.latency.estimated_latency_frames,
        status.descriptor.block_size_frames * status.descriptor.latency_blocks
    );
    assert!(status.latency.estimated_latency_seconds > 0.0);
    assert_eq!(status.latency.queued_samples, None);
    assert_eq!(status.latency.capacity_samples, None);
    assert!(status.diagnostics.is_empty());
}
