use super::{
    generate_initial_ragdoll_profile, prepare_ragdoll_profile_edits, RagdollBoneEdit,
    RagdollBoneProperty, RagdollProfileEditError, RagdollSkeletonBone,
};
use zircon_plugin_physics_runtime::{RagdollProfile, RagdollProfileError};
use zircon_runtime::core::framework::physics::PhysicsColliderShape;
use zircon_runtime::core::framework::scene::physics::PhysicsJointConstraintMetadata;
use zircon_runtime::core::math::{Real, Transform, Vec3};

const ROOT: &str = "Rig";
const CHILD: &str = "Rig/骨骼";

fn profile() -> RagdollProfile {
    // Unit data uses the actual generator; this is not an imported preview asset.
    generate_initial_ragdoll_profile(
        "hero",
        &[
            RagdollSkeletonBone::new(ROOT, None, Transform::default()),
            RagdollSkeletonBone::new(
                CHILD,
                Some(ROOT),
                Transform::from_translation(Vec3::new(0.0, 0.6, 0.0)),
            ),
        ],
    )
    .expect("valid unit skeleton")
}

fn edit(path: &str, property: RagdollBoneProperty) -> RagdollBoneEdit {
    RagdollBoneEdit {
        bone_path: path.to_owned(),
        property,
    }
}

#[test]
fn ragdoll_profile_edit_prepares_all_five_properties_in_one_candidate() {
    let source = profile();
    let constraint = PhysicsJointConstraintMetadata {
        angular_limits: [Some([-0.4, 0.4]), None, None],
        break_force: Some(30.0),
        ..PhysicsJointConstraintMetadata::default()
    };
    let shape = PhysicsColliderShape::Sphere { radius: 0.3 };
    let offset = Transform::from_translation(Vec3::new(0.1, 0.2, 0.3));
    let candidate = prepare_ragdoll_profile_edits(
        &source,
        &[
            edit(CHILD, RagdollBoneProperty::Shape(shape.clone())),
            edit(ROOT, RagdollBoneProperty::Mass(2.0)),
            edit(CHILD, RagdollBoneProperty::BodyOffset(offset)),
            edit(CHILD, RagdollBoneProperty::Constraint(constraint.clone())),
            edit(CHILD, RagdollBoneProperty::BlendWeight(0.5)),
        ],
    )
    .expect("valid batch")
    .expect("one changed candidate");
    let mut expected = source.clone();
    expected.bones[0].mass = 2.0;
    expected.bones[1].shape = shape;
    expected.bones[1].body_offset = offset;
    expected.bones[1].constraint = constraint;
    expected.bones[1].blend_weight = 0.5;
    assert_eq!(candidate, expected);
    assert_eq!(source, profile());
    assert!(candidate.validate().is_ok());
}

#[test]
fn ragdoll_profile_edit_empty_and_net_reversal_need_no_history_entry() {
    let source = profile();
    assert_eq!(prepare_ragdoll_profile_edits(&source, &[]), Ok(None));
    assert_eq!(
        prepare_ragdoll_profile_edits(
            &source,
            &[
                edit(ROOT, RagdollBoneProperty::Mass(2.0)),
                edit(ROOT, RagdollBoneProperty::Mass(1.0)),
                edit(
                    CHILD,
                    RagdollBoneProperty::Shape(source.bones[1].shape.clone())
                ),
                edit(
                    CHILD,
                    RagdollBoneProperty::BodyOffset(source.bones[1].body_offset)
                ),
                edit(
                    CHILD,
                    RagdollBoneProperty::Constraint(source.bones[1].constraint.clone())
                ),
                edit(CHILD, RagdollBoneProperty::BlendWeight(1.0)),
            ],
        ),
        Ok(None)
    );
}

#[test]
fn ragdoll_profile_edit_missing_later_target_rejects_the_whole_batch() {
    let source = profile();
    let before = source.clone();
    assert_eq!(
        prepare_ragdoll_profile_edits(
            &source,
            &[
                edit(ROOT, RagdollBoneProperty::Mass(2.0)),
                edit("Missing", RagdollBoneProperty::Mass(3.0)),
            ],
        ),
        Err(RagdollProfileEditError::MissingBone {
            bone_path: "Missing".to_owned()
        })
    );
    assert_eq!(source, before);
}

#[test]
fn ragdoll_profile_edit_never_resolves_leaf_alias_case_or_trimmed_path() {
    let source = profile();
    for path in ["骨骼", "rig/骨骼", "Rig/骨骼 ", " Rig/骨骼"] {
        assert_eq!(
            prepare_ragdoll_profile_edits(&source, &[edit(path, RagdollBoneProperty::Mass(2.0))]),
            Err(RagdollProfileEditError::MissingBone {
                bone_path: path.to_owned()
            })
        );
    }
}

#[test]
fn ragdoll_profile_edit_exact_chinese_long_path_survives_duplicate_leaf_names() {
    let mut source = profile();
    let long_path = format!("Rig/{}/骨骼", "长层级".repeat(40));
    let mut other = source.bones[1].clone();
    other.bone_path = long_path.clone();
    source.bones.push(other);
    source.validate().expect("unique exact authored paths");
    let candidate =
        prepare_ragdoll_profile_edits(&source, &[edit(&long_path, RagdollBoneProperty::Mass(4.0))])
            .expect("valid exact path")
            .expect("change");
    assert_eq!(candidate.bones[2].bone_path, long_path);
    assert_eq!(candidate.bones[2].mass, 4.0);
    assert_eq!(candidate.bones[0], source.bones[0]);
    assert_eq!(candidate.bones[1], source.bones[1]);
}

#[test]
fn ragdoll_profile_edit_preserves_original_slots_even_with_later_parent() {
    let mut source = profile();
    source.bones.swap(0, 1);
    let candidate =
        prepare_ragdoll_profile_edits(&source, &[edit(CHILD, RagdollBoneProperty::Mass(3.0))])
            .expect("valid child-before-parent input")
            .expect("change");
    assert_eq!(candidate.id, source.id);
    assert_eq!(candidate.bones[0].bone_path, CHILD);
    assert_eq!(candidate.bones[0].parent_bone_path.as_deref(), Some(ROOT));
    assert_eq!(candidate.bones[0].mass, 3.0);
    assert_eq!(candidate.bones[1], source.bones[1]);
}

#[test]
fn ragdoll_profile_edit_rejects_invalid_mass_without_partial_candidate() {
    let source = profile();
    for value in [0.0, -1.0, Real::NAN, Real::INFINITY] {
        assert_eq!(
            prepare_ragdoll_profile_edits(
                &source,
                &[
                    edit(ROOT, RagdollBoneProperty::Mass(2.0)),
                    edit(CHILD, RagdollBoneProperty::Mass(value)),
                ],
            ),
            Err(RagdollProfileEditError::InvalidProfile(
                RagdollProfileError::InvalidMass {
                    bone_path: CHILD.to_owned()
                }
            ))
        );
    }
    assert_eq!(source, profile());
}

#[test]
fn ragdoll_profile_edit_rejects_nonfinite_body_offset() {
    let source = profile();
    let mut translation = Transform::default();
    translation.translation.x = Real::NAN;
    let mut rotation = Transform::default();
    rotation.rotation.w = Real::INFINITY;
    let mut scale = Transform::default();
    scale.scale.z = Real::NAN;
    for value in [translation, rotation, scale] {
        assert_eq!(
            prepare_ragdoll_profile_edits(
                &source,
                &[edit(CHILD, RagdollBoneProperty::BodyOffset(value))]
            ),
            Err(RagdollProfileEditError::InvalidProfile(
                RagdollProfileError::InvalidTransform {
                    bone_path: CHILD.to_owned()
                }
            ))
        );
    }
}

#[test]
fn ragdoll_profile_edit_rejects_bad_shape_and_compound_child_transform() {
    let source = profile();
    let shapes = [
        PhysicsColliderShape::Sphere { radius: 0.0 },
        PhysicsColliderShape::Box {
            half_extents: [0.1, 0.0, 0.3],
        },
        PhysicsColliderShape::Compound {
            children: vec![(
                Transform {
                    scale: Vec3::splat(2.0),
                    ..Transform::default()
                },
                Box::new(PhysicsColliderShape::Sphere { radius: 0.2 }),
            )],
        },
    ];
    for value in shapes {
        assert_eq!(
            prepare_ragdoll_profile_edits(
                &source,
                &[edit(CHILD, RagdollBoneProperty::Shape(value))]
            ),
            Err(RagdollProfileEditError::InvalidProfile(
                RagdollProfileError::InvalidShape {
                    bone_path: CHILD.to_owned()
                }
            ))
        );
    }
}

#[test]
fn ragdoll_profile_edit_rejects_invalid_constraint() {
    let source = profile();
    let mut constraint = PhysicsJointConstraintMetadata::default();
    constraint.angular_limits[1] = Some([1.0, -1.0]);
    assert_eq!(
        prepare_ragdoll_profile_edits(
            &source,
            &[edit(CHILD, RagdollBoneProperty::Constraint(constraint))],
        ),
        Err(RagdollProfileEditError::InvalidProfile(
            RagdollProfileError::InvalidConstraint {
                bone_path: CHILD.to_owned()
            }
        ))
    );
}

#[test]
fn ragdoll_profile_edit_rejects_invalid_blend_weight() {
    let source = profile();
    for value in [-0.1, 1.1, Real::NAN, Real::INFINITY] {
        assert_eq!(
            prepare_ragdoll_profile_edits(
                &source,
                &[edit(CHILD, RagdollBoneProperty::BlendWeight(value))]
            ),
            Err(RagdollProfileEditError::InvalidProfile(
                RagdollProfileError::InvalidBlendWeight {
                    bone_path: CHILD.to_owned()
                }
            ))
        );
    }
}

#[test]
fn ragdoll_profile_edit_rejects_invalid_original_before_any_repair_or_noop() {
    let mut source = profile();
    source.bones[0].mass = 0.0;
    let before = source.clone();
    for edits in [vec![], vec![edit(ROOT, RagdollBoneProperty::Mass(1.0))]] {
        assert_eq!(
            prepare_ragdoll_profile_edits(&source, &edits),
            Err(RagdollProfileEditError::InvalidProfile(
                RagdollProfileError::InvalidMass {
                    bone_path: ROOT.to_owned()
                }
            ))
        );
    }
    assert_eq!(source, before);
}
