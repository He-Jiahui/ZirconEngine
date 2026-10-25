use super::{
    generate_initial_ragdoll_profile, prepare_ragdoll_body_translations, RagdollBodyTranslation,
    RagdollBodyTranslationError, RagdollProfileEditError, RagdollSkeletonBone,
};
use zircon_plugin_physics_runtime::{RagdollProfile, RagdollProfileError};
use zircon_runtime::core::math::{AffineInverseError, Mat4, Quat, Transform, Vec3};

const ROOT: &str = "Rig";
const CHILD: &str = "Rig/骨骼";

fn profile() -> RagdollProfile {
    generate_initial_ragdoll_profile(
        "body-drag-test",
        &[
            RagdollSkeletonBone::new(ROOT, None, Transform::default()),
            RagdollSkeletonBone::new(CHILD, Some(ROOT), Transform::from_translation(Vec3::Y)),
        ],
    )
    .expect("actual generator accepts unit skeleton")
}

fn drag(path: &str, frame: Mat4, delta: Vec3) -> RagdollBodyTranslation {
    RagdollBodyTranslation {
        bone_path: path.to_owned(),
        bone_to_world: frame,
        world_delta: delta,
    }
}

fn near(actual: Vec3, expected: Vec3) {
    assert!(
        (actual - expected).length() < 0.00001,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn ragdoll_body_translation_uses_bone_vector_frame_and_preserves_other_properties() {
    let mut source = profile();
    source.bones[1].body_offset = Transform::from_translation(Vec3::new(0.1, 0.2, 0.3))
        .with_rotation(Quat::from_rotation_y(0.4))
        .with_scale(Vec3::splat(0.7));
    let original = source.clone();
    let frame = Transform::from_translation(Vec3::new(100.0, 200.0, 300.0))
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(2.0, 3.0, 4.0))
        .matrix();
    let result =
        prepare_ragdoll_body_translations(&source, &[drag(CHILD, frame, Vec3::new(0.0, 2.0, 0.0))])
            .expect("valid drag")
            .expect("changed profile");
    near(
        result.bones[1].body_offset.translation,
        original.bones[1].body_offset.translation + Vec3::X,
    );
    let mut expected = original.clone();
    expected.bones[1].body_offset.translation = result.bones[1].body_offset.translation;
    assert_eq!(result, expected);
    assert_eq!(source, original);
}

#[test]
fn ragdoll_body_translation_multi_selection_resolves_each_frame_in_one_candidate() {
    let source = profile();
    let reflected = Mat4::from_scale(Vec3::new(-2.0, 3.0, 4.0));
    let delta = Vec3::new(2.0, 3.0, 4.0);
    let result = prepare_ragdoll_body_translations(
        &source,
        &[
            drag(ROOT, Mat4::IDENTITY, delta),
            drag(CHILD, reflected, delta),
        ],
    )
    .unwrap()
    .unwrap();
    near(result.bones[0].body_offset.translation, delta);
    near(
        result.bones[1].body_offset.translation,
        Vec3::new(-1.0, 1.0, 1.0),
    );
    assert!(result.validate().is_ok());
    assert_eq!(source, profile());
}

#[test]
fn ragdoll_body_translation_empty_and_zero_drag_need_no_history_entry() {
    let source = profile();
    assert_eq!(prepare_ragdoll_body_translations(&source, &[]), Ok(None));
    assert_eq!(
        prepare_ragdoll_body_translations(&source, &[drag(CHILD, Mat4::IDENTITY, Vec3::ZERO)]),
        Ok(None)
    );
}

#[test]
fn ragdoll_body_translation_missing_or_duplicate_later_target_rejects_whole_batch() {
    let source = profile();
    for path in ["骨骼", "rig/骨骼", "Rig/骨骼 ", "missing"] {
        assert_eq!(
            prepare_ragdoll_body_translations(
                &source,
                &[
                    drag(ROOT, Mat4::IDENTITY, Vec3::X),
                    drag(path, Mat4::IDENTITY, Vec3::Y),
                ]
            ),
            Err(RagdollBodyTranslationError::Edit(
                RagdollProfileEditError::MissingBone {
                    bone_path: path.to_owned()
                }
            ))
        );
    }
    assert_eq!(
        prepare_ragdoll_body_translations(
            &source,
            &[
                drag(CHILD, Mat4::IDENTITY, Vec3::X),
                drag(CHILD, Mat4::IDENTITY, Vec3::Y),
            ]
        ),
        Err(RagdollBodyTranslationError::DuplicateTarget {
            bone_path: CHILD.to_owned()
        })
    );
    assert_eq!(source, profile());
}

#[test]
fn ragdoll_body_translation_rejects_nonfinite_nonaffine_and_singular_frames() {
    let source = profile();
    let mut nonaffine = Mat4::IDENTITY;
    nonaffine.x_axis.w = 0.5;
    let mut nonfinite = Mat4::IDENTITY;
    nonfinite.w_axis.x = f32::NAN;
    for (matrix, expected) in [
        (nonaffine, AffineInverseError::NonAffineInput),
        (nonfinite, AffineInverseError::NonFiniteInput),
    ] {
        assert_eq!(
            prepare_ragdoll_body_translations(&source, &[drag(CHILD, matrix, Vec3::X)]),
            Err(RagdollBodyTranslationError::InvalidBoneFrame {
                bone_path: CHILD.to_owned(),
                source: expected
            })
        );
    }
    assert!(matches!(
        prepare_ragdoll_body_translations(
            &source,
            &[drag(CHILD, Mat4::from_scale(Vec3::ZERO), Vec3::X)]
        ),
        Err(RagdollBodyTranslationError::InvalidBoneFrame { .. })
    ));
    assert_eq!(
        prepare_ragdoll_body_translations(
            &source,
            &[drag(CHILD, Mat4::IDENTITY, Vec3::splat(f32::INFINITY))]
        ),
        Err(RagdollBodyTranslationError::NonFiniteDelta {
            bone_path: CHILD.to_owned()
        })
    );
    assert_eq!(source, profile());
}

#[test]
fn ragdoll_body_translation_world_reconstruction_includes_inherited_affine_basis() {
    let source = profile();
    let mut sheared = Mat4::IDENTITY;
    sheared.y_axis.x = 0.4;
    sheared.w_axis.x = 20.0;
    let delta = Vec3::new(0.6, -0.3, 0.8);
    let result = prepare_ragdoll_body_translations(&source, &[drag(CHILD, sheared, delta)])
        .unwrap()
        .unwrap();
    let before = sheared.transform_point3(source.bones[1].body_offset.translation);
    let after = sheared.transform_point3(result.bones[1].body_offset.translation);
    near(after - before, delta);
}

#[test]
fn ragdoll_body_translation_finite_overflow_rejects_later_offset_without_source_change() {
    let mut source = profile();
    source.bones[1].body_offset.translation.x = f32::MAX;
    let original = source.clone();
    assert_eq!(
        prepare_ragdoll_body_translations(
            &source,
            &[
                drag(ROOT, Mat4::IDENTITY, Vec3::X),
                drag(CHILD, Mat4::IDENTITY, Vec3::new(f32::MAX, 0.0, 0.0)),
            ]
        ),
        Err(RagdollBodyTranslationError::NonFiniteOffset {
            bone_path: CHILD.to_owned()
        })
    );
    assert_eq!(source, original);
}

#[test]
fn ragdoll_body_translation_validates_source_even_for_an_empty_gesture() {
    let mut source = profile();
    source.bones[0].mass = 0.0;
    assert_eq!(
        prepare_ragdoll_body_translations(&source, &[]),
        Err(RagdollBodyTranslationError::Edit(
            RagdollProfileEditError::InvalidProfile(RagdollProfileError::InvalidMass {
                bone_path: ROOT.to_owned()
            })
        ))
    );
}
