use super::*;
use zircon_runtime::rhi::{DeviceGeneration, DeviceId};

fn epoch(generation: u64) -> RuntimePrepareDeviceEpoch {
    RuntimePrepareDeviceEpoch::new(DeviceId::new(17), DeviceGeneration::new(generation))
}

#[test]
fn device_epoch_change_releases_persistent_particle_gpu_state() {
    let mut owner = ParticleGpuRuntimeOwner::default();
    assert!(owner.activate_device_epoch(epoch(3)));
    owner.aggregate_asset = Some(ParticleSystemAsset::new("device-epoch-test"));
    owner.aggregate_executed = true;
    owner.next_frame_transaction_id = 41;

    assert!(!owner.activate_device_epoch(epoch(3)));
    assert!(owner.aggregate_asset.is_some());
    assert!(owner.aggregate_executed);

    assert!(owner.activate_device_epoch(epoch(4)));
    assert_eq!(owner.active_device_epoch, Some(epoch(4)));
    assert!(owner.states.is_empty());
    assert!(owner.aggregate_asset.is_none());
    assert!(owner.aggregate_backend.is_none());
    assert!(!owner.aggregate_executed);
    assert!(owner.pending_frame.is_none());
    assert_eq!(owner.next_frame_transaction_id, 41);
}
