use std::time::{Duration, Instant};

use super::{asset_maintenance_frame_update, AssetMaintenanceFrameUpdate};

#[test]
fn empty_refresh_preserves_active_scene_retry_deadline() {
    let retry_deadline = Instant::now() + Duration::from_millis(128);

    assert_eq!(
        asset_maintenance_frame_update(false, None, Some(retry_deadline)),
        AssetMaintenanceFrameUpdate::At(retry_deadline)
    );
}

#[test]
fn earliest_asset_owner_deadline_drives_the_shared_maintenance_slot() {
    let now = Instant::now();
    let accumulator_deadline = now + Duration::from_millis(32);
    let retry_deadline = now + Duration::from_millis(128);

    assert_eq!(
        asset_maintenance_frame_update(false, Some(accumulator_deadline), Some(retry_deadline)),
        AssetMaintenanceFrameUpdate::At(accumulator_deadline)
    );
}
