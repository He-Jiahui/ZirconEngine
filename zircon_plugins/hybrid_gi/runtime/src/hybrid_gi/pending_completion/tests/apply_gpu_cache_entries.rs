use super::*;
use zircon_runtime::core::math::Vec3;

#[test]
fn runtime98_gpu_cache_membership_preserves_first_slot_and_evicts_absent_residents() {
    let mut state = HybridGiRuntimeState::default();
    state.seed_runtime_probe_scene_data_for_test([
        (1, Vec3::ZERO, 1.0, None, 64),
        (2, Vec3::ZERO, 1.0, None, 64),
        (3, Vec3::ZERO, 1.0, None, 64),
        (4, Vec3::ZERO, 1.0, None, 64),
    ]);
    state.insert_resident_probe_slot(1, 1);

    state.apply_gpu_cache_entries(&[(2, 7), (2, 9), (3, 8), (4, 8), (99, 11)]);

    assert_eq!(state.probe_slot(1), None);
    assert_eq!(state.probe_slot(2), Some(7));
    assert_eq!(state.probe_slot(3), None);
    assert_eq!(state.probe_slot(4), Some(8));
    assert_eq!(state.probe_slot(99), None);
}
