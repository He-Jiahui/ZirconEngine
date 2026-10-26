use super::{RagdollBoneProfile, RagdollProfile, RagdollProfileError};
use zircon_runtime::core::framework::physics::PhysicsColliderShape;
use zircon_runtime::core::framework::scene::physics::PhysicsJointConstraintMetadata;
use zircon_runtime::core::math::{Real, Transform, Vec3};

// Unit data exercises the admission contract, not an installed character asset.
fn profile() -> RagdollProfile {
    RagdollProfile {
        id: "hero".to_owned(),
        bones: vec![RagdollBoneProfile {
            bone_path: "Rig".to_owned(),
            parent_bone_path: None,
            shape: PhysicsColliderShape::Capsule {
                radius: 0.15,
                half_height: 0.2,
            },
            mass: 1.0,
            body_offset: Transform::default(),
            constraint: PhysicsJointConstraintMetadata::default(),
            blend_weight: 1.0,
        }],
    }
}

fn assert_invalid_constraint(constraint: PhysicsJointConstraintMetadata) {
    let mut source = profile();
    source.bones[0].constraint = constraint;
    assert_eq!(
        source.validate(),
        Err(RagdollProfileError::InvalidConstraint {
            bone_path: "Rig".to_owned()
        })
    );
}

#[test]
fn ragdoll_profile_validation_rejects_unordered_or_nonfinite_constraint_limits() {
    for limit in [[2.0, -2.0], [Real::NAN, 1.0], [0.0, Real::INFINITY]] {
        for axis in 0..3 {
            let mut constraint = PhysicsJointConstraintMetadata::default();
            constraint.linear_limits[axis] = Some(limit);
            assert_invalid_constraint(constraint);
            let mut constraint = PhysicsJointConstraintMetadata::default();
            constraint.angular_limits[axis] = Some(limit);
            assert_invalid_constraint(constraint);
        }
    }
}

#[test]
fn ragdoll_profile_validation_rejects_invalid_constraint_drives() {
    for angular in [false, true] {
        for field in 0..5 {
            let mut constraint = PhysicsJointConstraintMetadata::default();
            let drive = if angular {
                &mut constraint.angular_drives[2]
            } else {
                &mut constraint.linear_drives[1]
            };
            match field {
                0 => drive.target_position = Real::NAN,
                1 => drive.target_velocity = Real::INFINITY,
                2 => drive.stiffness = -1.0,
                3 => drive.damping = -1.0,
                4 => drive.max_force = -1.0,
                _ => unreachable!(),
            }
            assert_invalid_constraint(constraint);
        }
    }
}

#[test]
fn ragdoll_profile_validation_rejects_invalid_constraint_thresholds() {
    for value in [-1.0, Real::NAN, Real::INFINITY] {
        for field in 0..4 {
            let mut constraint = PhysicsJointConstraintMetadata::default();
            match field {
                0 => constraint.break_force = Some(value),
                1 => constraint.break_torque = Some(value),
                2 => constraint.projection_linear_tolerance = Some(value),
                3 => constraint.projection_angular_tolerance = Some(value),
                _ => unreachable!(),
            }
            assert_invalid_constraint(constraint);
        }
    }
}

#[test]
fn ragdoll_profile_validation_rejects_nested_compound_child_transforms() {
    let mut translation = Transform::default();
    translation.translation.x = Real::NAN;
    let mut rotation = Transform::default();
    rotation.rotation.w = Real::INFINITY;
    let mut scale = Transform::default();
    scale.scale.y = Real::NAN;
    let mut scaled = Transform::default();
    scaled.scale = Vec3::splat(2.0);
    for transform in [translation, rotation, scale, scaled] {
        let mut source = profile();
        source.bones[0].shape = PhysicsColliderShape::Compound {
            children: vec![(
                Transform::default(),
                Box::new(PhysicsColliderShape::Compound {
                    children: vec![(
                        transform,
                        Box::new(PhysicsColliderShape::Sphere { radius: 0.2 }),
                    )],
                }),
            )],
        };
        assert_eq!(
            source.validate(),
            Err(RagdollProfileError::InvalidShape {
                bone_path: "Rig".to_owned()
            })
        );
    }
}

#[test]
fn ragdoll_profile_validation_keeps_strict_box_extents_inside_compounds() {
    let mut source = profile();
    source.bones[0].shape = PhysicsColliderShape::Compound {
        children: vec![(
            Transform::default(),
            Box::new(PhysicsColliderShape::Box {
                half_extents: [0.2, 0.0, 0.3],
            }),
        )],
    };
    assert_eq!(
        source.validate(),
        Err(RagdollProfileError::InvalidShape {
            bone_path: "Rig".to_owned()
        })
    );
}

#[test]
fn ragdoll_profile_validation_parser_uses_the_same_constraint_admission() {
    let mut source = profile();
    source.bones[0].constraint.linear_limits[0] = Some([-0.3, 0.3]);
    source.bones[0].constraint.angular_limits[2] = Some([-0.8, 0.8]);
    source.bones[0].constraint.angular_drives[2].target_position = -0.2;
    source.bones[0].constraint.angular_drives[2].max_force = 20.0;
    source.bones[0].constraint.break_force = Some(0.0);
    let valid_toml = toml::to_string(&source).expect("serialize valid profile");
    assert_eq!(RagdollProfile::from_toml(&valid_toml), Ok(source.clone()));

    source.bones[0].constraint.angular_limits[2] = Some([0.8, -0.8]);
    let invalid_toml = toml::to_string(&source).expect("serialize invalid numeric policy");
    let error = RagdollProfile::from_toml(&invalid_toml).expect_err("reject invalid limits");
    assert_eq!(
        error,
        RagdollProfileError::InvalidConstraint {
            bone_path: "Rig".to_owned()
        }
    );
    assert_eq!(error.to_string(), "ragdoll bone Rig has invalid constraint");
}
