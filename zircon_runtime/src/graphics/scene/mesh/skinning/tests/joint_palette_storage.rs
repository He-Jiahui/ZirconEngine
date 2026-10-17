use super::*;

const BENCHMARK_INSTANCE_COUNT: usize = 1_000;

#[test]
fn thousand_draws_share_one_compact_palette_and_release_it() {
    let payload =
        Arc::new(SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64]).unwrap());
    let weak = Arc::downgrade(&payload);
    let payloads = vec![Some(payload.clone()); BENCHMARK_INSTANCE_COUNT * 2];
    assert_eq!(
        std::mem::size_of_val(payloads.as_slice()),
        BENCHMARK_INSTANCE_COUNT * 2 * std::mem::size_of::<usize>()
    );
    assert_eq!(
        std::mem::size_of_val(payload.active_joint_matrices()),
        64 * 64
    );
    assert!(payloads
        .iter()
        .all(|entry| Arc::ptr_eq(entry.as_ref().unwrap(), &payload)));
    drop(payload);
    assert!(weak.upgrade().is_some());
    drop(payloads);
    assert!(weak.upgrade().is_none());
}

#[test]
fn palette_allocates_only_active_matrices_and_preserves_the_joint_limit() {
    for count in [0, 1, 64, 256] {
        let matrices = vec![Mat4::from_translation(crate::core::math::Vec3::X); count];
        let palette = SkinnedMeshJointPaletteStorage::from_matrices(&matrices).unwrap();
        assert_eq!(palette.joint_count() as usize, count);
        assert_eq!(
            std::mem::size_of_val(palette.active_joint_matrices()),
            count * 64
        );
        assert!(palette
            .active_joint_matrices()
            .iter()
            .all(|matrix| *matrix == matrices[0].to_cols_array_2d()));
    }
    assert!(SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 257]).is_err());
}

#[test]
fn a_new_pose_does_not_mutate_the_previous_snapshot() {
    let previous =
        Arc::new(SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY]).unwrap());
    let retained = previous.clone();
    let current = Arc::new(
        SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::from_translation(
            crate::core::math::Vec3::X,
        )])
        .unwrap(),
    );
    assert_eq!(
        retained.active_joint_matrices(),
        &[Mat4::IDENTITY.to_cols_array_2d()]
    );
    assert!(!Arc::ptr_eq(&retained, &current));
}
