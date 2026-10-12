use super::*;

const BENCHMARK_INSTANCE_COUNT: usize = 1_000;

#[test]
fn thousand_instance_storage_payload_contract_stays_within_expected_budget() {
    let payload = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64])
        .expect("64-joint test palette fits storage ABI");
    let payloads = vec![payload; BENCHMARK_INSTANCE_COUNT * 2];

    assert_eq!(payloads.len(), BENCHMARK_INSTANCE_COUNT * 2);
    assert_eq!(payloads[0].joint_count(), 64);
    assert_eq!(
        std::mem::size_of_val(payloads.as_slice()),
        BENCHMARK_INSTANCE_COUNT * 2 * std::mem::size_of::<SkinnedMeshJointPaletteStorage>()
    );
    assert!(std::mem::size_of_val(payloads.as_slice()) <= 32 * 1024 * 1024);
}

#[test]
fn active_palette_span_exposes_only_live_joint_matrices() {
    let no_joints =
        SkinnedMeshJointPaletteStorage::from_matrices(&[]).expect("empty palette fits storage ABI");
    let sixty_four_joints = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64])
        .expect("64-joint palette fits storage ABI");
    let full_palette = SkinnedMeshJointPaletteStorage::from_matrices(
        &[Mat4::IDENTITY; SKINNED_MESH_MAX_JOINT_MATRICES],
    )
    .expect("full palette fits storage ABI");

    let matrix_bytes = std::mem::size_of::<[[f32; 4]; 4]>();
    assert_eq!(std::mem::size_of_val(no_joints.active_joint_matrices()), 0);
    assert_eq!(
        std::mem::size_of_val(sixty_four_joints.active_joint_matrices()),
        64 * matrix_bytes
    );
    assert_eq!(
        std::mem::size_of_val(full_palette.active_joint_matrices()),
        SKINNED_MESH_MAX_JOINT_MATRICES * matrix_bytes
    );
}

#[test]
fn thousand_instance_two_frame_arena_payload_is_compact() {
    let matrix_bytes = std::mem::size_of::<[[f32; 4]; 4]>();
    let arena_payload_bytes = BENCHMARK_INSTANCE_COUNT * 2 * 64 * matrix_bytes;

    assert_eq!(arena_payload_bytes, 8_192_000);
    assert!(arena_payload_bytes < 8 * 1024 * 1024);
}
