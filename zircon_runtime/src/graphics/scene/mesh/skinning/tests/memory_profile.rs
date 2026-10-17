use super::*;
use crate::memory_validation::{live_bytes, measure};
// Layout from base-head f28ba39: fixed palette plus a parameter vector.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct LegacyPalette {
    matrices: [[[f32; 4]; 4]; 256],
    params: [u32; 4],
}
#[derive(Clone, Copy)]
struct LegacyDraw {
    current: Option<LegacyPalette>,
    previous: Option<LegacyPalette>,
}
#[test]
fn memory_profile_palette_draws() {
    const DRAWS: usize = 2000;
    let baseline = live_bytes();
    let (legacy, old) = measure(|| {
        let palette = LegacyPalette {
            matrices: [Mat4::IDENTITY.to_cols_array_2d(); 256],
            params: [64, 0, 0, 0],
        };
        vec![
            LegacyDraw {
                current: Some(palette),
                previous: Some(palette)
            };
            DRAWS
        ]
    });
    assert_eq!(legacy[0].current.unwrap().params[0], 64);
    assert_eq!(
        legacy[0].previous.unwrap().matrices[0],
        Mat4::IDENTITY.to_cols_array_2d()
    );
    old.report("2000_draws_64_joints_two_poses", "fixed_inline_baseline");
    drop(legacy);
    assert_eq!(live_bytes(), baseline);
    let (shared, new) = measure(|| {
        let current =
            Arc::new(SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64]).unwrap());
        let previous =
            Arc::new(SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64]).unwrap());
        vec![(Some(current), Some(previous)); DRAWS]
    });
    assert!(new.live_bytes < old.live_bytes / 100);
    new.report("2000_draws_64_joints_two_poses", "shared_exact_snapshots");
    drop(shared);
    assert_eq!(live_bytes(), baseline);
}
