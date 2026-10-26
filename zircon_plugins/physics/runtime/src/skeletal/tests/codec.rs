use super::{RagdollBoneProfile, RagdollProfile, RagdollProfileEncodeError};
use zircon_runtime::core::framework::physics::PhysicsColliderShape;
use zircon_runtime::core::framework::scene::physics::PhysicsJointConstraintMetadata;
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};
use zircon_runtime::core::resource::{AssetReference, AssetUuid, ResourceLocator};

// Unit values exercise the real codec; they do not register a character/resource producer.
fn profile() -> RagdollProfile {
    RagdollProfile {
        id: "character".to_owned(),
        bones: vec![RagdollBoneProfile {
            bone_path: "Rig/Root".to_owned(),
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

fn round_trip(source: &RagdollProfile) -> String {
    let original = source.clone();
    let text = source.to_toml().expect("encode validated profile");
    let restored = RagdollProfile::from_toml(&text).expect("decode existing profile schema");
    assert_eq!(&restored, source);
    assert_eq!(source, &original);
    text
}

fn reference(label: &str, locator: &str) -> AssetReference {
    // Intentionally distinct from the locator-derived identity: the writer must retain both.
    AssetReference::new(
        AssetUuid::from_stable_label(label),
        ResourceLocator::parse(locator).unwrap(),
    )
}

#[test]
fn minimal_existing_toml_defaults_round_trip_without_changing_profile_identity() {
    let source = RagdollProfile::from_toml(
        r#"
        id = "existing"
        [[bones]]
        bone_path = "Rig/Root"
        [bones.shape]
        kind = "sphere"
        radius = 0.25
    "#,
    )
    .unwrap();
    let text = round_trip(&source);
    assert_eq!(source.id, "existing");
    assert_eq!(source.bones[0].mass, 1.0);
    assert_eq!(source.bones[0].blend_weight, 1.0);
    assert_eq!(source.to_toml().unwrap(), text);
}

#[test]
fn unicode_escaped_long_paths_and_parent_after_child_keep_exact_order_and_text() {
    let mut source = profile();
    source.id = "角色\"Profile\"\t#1".to_owned();
    let root = " 角色/根骨 ";
    let mut child = source.bones[0].clone();
    source.bones[0].bone_path = root.to_owned();
    child.bone_path = format!("{root}/Head\n\"{}\"", "长名称".repeat(256));
    child.parent_bone_path = Some(root.to_owned());
    source.bones.insert(0, child);
    let mut other = source.bones[1].clone();
    other.bone_path = "Other/Head".to_owned();
    source.bones.push(other);
    round_trip(&source);
    assert_eq!(source.bones[0].parent_bone_path.as_deref(), Some(root));
    assert_eq!(source.bones[1].bone_path, root);
}

#[test]
fn every_existing_collision_shape_preserves_geometry_transforms_and_asset_reference_pairs() {
    let child_frame = Transform {
        translation: Vec3::new(-0.5, 2.0, 0.25),
        rotation: Quat::from_rotation_z(0.3),
        scale: Vec3::ONE,
    };
    let shapes = vec![
        PhysicsColliderShape::Box {
            half_extents: [0.5, 0.75, 0.125],
        },
        PhysicsColliderShape::Sphere { radius: 0.125 },
        PhysicsColliderShape::Capsule {
            radius: 0.15,
            half_height: 0.0,
        },
        PhysicsColliderShape::Cylinder {
            radius: 0.25,
            half_height: 0.75,
        },
        PhysicsColliderShape::ConvexHull {
            points: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        },
        PhysicsColliderShape::TriangleMesh {
            mesh: reference(
                "unit-mesh-persistent-id",
                "res://characters/moved/collider.zrmesh",
            ),
        },
        PhysicsColliderShape::HeightField {
            resolution: [3, 4],
            heights: reference("unit-height-persistent-id", "res://characters/heights.bin"),
        },
        PhysicsColliderShape::Compound {
            children: vec![
                (
                    child_frame,
                    Box::new(PhysicsColliderShape::Sphere { radius: 0.3 }),
                ),
                (
                    Transform::default(),
                    Box::new(PhysicsColliderShape::Compound {
                        children: vec![(
                            child_frame,
                            Box::new(PhysicsColliderShape::Box {
                                half_extents: [0.2; 3],
                            }),
                        )],
                    }),
                ),
            ],
        },
    ];
    for shape in shapes {
        let mut source = profile();
        source.bones[0].shape = shape;
        round_trip(&source);
    }
}

#[test]
fn sparse_named_limits_and_all_drive_fields_retain_axis_slots() {
    let mut source = profile();
    let constraint = &mut source.bones[0].constraint;
    constraint.linear_limits = [Some([-0.3, 0.75]), None, Some([0.0, 0.0])];
    constraint.angular_limits = [None, Some([-0.5, 0.25]), Some([-1.0, 1.0])];
    for axis in 0..3 {
        let drive = &mut constraint.linear_drives[axis];
        drive.target_position = -(axis as Real) - 0.125;
        drive.target_velocity = 0.25 + axis as Real;
        drive.stiffness = 5.0 + axis as Real;
        drive.damping = 0.5 + axis as Real;
        drive.max_force = 20.0 + axis as Real;
        let drive = &mut constraint.angular_drives[axis];
        drive.target_position = 0.125 + axis as Real;
        drive.target_velocity = -0.25 - axis as Real;
        drive.stiffness = 10.0 + axis as Real;
        drive.damping = 1.5 + axis as Real;
        drive.max_force = 40.0 + axis as Real;
    }
    constraint.break_force = Some(100.0);
    constraint.break_torque = Some(5.0);
    constraint.projection_linear_tolerance = Some(0.125);
    constraint.projection_angular_tolerance = Some(0.25);
    round_trip(&source);
}

#[test]
fn optional_zero_thresholds_and_locked_axes_remain_distinct_from_unset() {
    let mut source = profile();
    let mut child = source.bones[0].clone();
    child.bone_path = "Rig/Root/Child".to_owned();
    child.parent_bone_path = Some(source.bones[0].bone_path.clone());
    child.constraint.break_force = Some(0.0);
    child.constraint.break_torque = Some(0.0);
    child.constraint.projection_linear_tolerance = Some(0.0);
    child.constraint.projection_angular_tolerance = Some(0.0);
    child.constraint.linear_limits[1] = Some([0.0, 0.0]);
    child.constraint.angular_limits[2] = Some([0.0, 0.0]);
    source.bones.push(child);
    let restored = RagdollProfile::from_toml(&round_trip(&source)).unwrap();
    assert_eq!(restored.bones[0].constraint.break_force, None);
    assert_eq!(restored.bones[1].constraint.break_force, Some(0.0));
    assert_eq!(restored.bones[0].constraint.linear_limits[1], None);
    assert_eq!(
        restored.bones[1].constraint.linear_limits[1],
        Some([0.0, 0.0])
    );
}

#[test]
fn finite_numeric_extremes_body_trs_and_negative_zero_do_not_get_normalized() {
    for mass in [Real::MIN_POSITIVE, 0.12345679, Real::MAX] {
        let mut source = profile();
        source.bones[0].mass = mass;
        source.bones[0].blend_weight = 0.625;
        source.bones[0].body_offset = Transform {
            translation: Vec3::new(1.2345679, -0.0, -27.125),
            rotation: Quat::from_rotation_x(0.25),
            scale: Vec3::new(-2.0, 0.125, 3.0),
        };
        let restored = RagdollProfile::from_toml(&round_trip(&source)).unwrap();
        assert_eq!(restored.bones[0].mass.to_bits(), mass.to_bits());
        assert_eq!(
            restored.bones[0].body_offset.translation.y.to_bits(),
            (-0.0 as Real).to_bits()
        );
    }
}

#[test]
fn invalid_profiles_return_original_admission_error_before_encoding() {
    let mut cases = Vec::new();
    for field in 0..10 {
        let mut source = profile();
        match field {
            0 => source.id.clear(),
            1 => source.bones.clear(),
            2 => source.bones[0].mass = Real::NAN,
            3 => source.bones[0].blend_weight = Real::INFINITY,
            4 => source.bones[0].shape = PhysicsColliderShape::Sphere { radius: 0.0 },
            5 => source.bones[0].body_offset.translation.x = Real::NAN,
            6 => source.bones[0].constraint.linear_limits[0] = Some([2.0, 1.0]),
            7 => source.bones[0].parent_bone_path = Some("missing".to_owned()),
            8 => source.bones.push(source.bones[0].clone()),
            _ => source.bones[0].constraint.angular_drives[2].max_force = -1.0,
        }
        cases.push(source);
    }
    for source in cases {
        let expected = source.validate().unwrap_err();
        let before = format!("{source:?}");
        match source.to_toml() {
            Err(RagdollProfileEncodeError::Profile(error)) => assert_eq!(error, expected),
            result => panic!("expected original admission error, found {result:?}"),
        }
        assert_eq!(format!("{source:?}"), before);
    }
}

#[test]
fn legacy_axis_arrays_reencode_as_the_existing_named_axis_schema_without_data_loss() {
    let source = RagdollProfile::from_toml(
        r#"
        id = "legacy"
        [[bones]]
        bone_path = "Rig/Root"
        [bones.shape]
        kind = "sphere"
        radius = 0.25
        [bones.constraint]
        linear_limits = [[-1.0, 1.0], [-2.0, 2.0], [-3.0, 3.0]]
        angular_limits = [[-0.1, 0.1], [-0.2, 0.2], [0.0, 0.0]]
    "#,
    )
    .unwrap();
    round_trip(&source);
    assert_eq!(
        source.bones[0].constraint.linear_limits[2],
        Some([-3.0, 3.0])
    );
    assert_eq!(
        source.bones[0].constraint.angular_limits[2],
        Some([0.0, 0.0])
    );
}

#[test]
fn unknown_root_or_bone_fields_are_rejected_before_a_typed_save_can_discard_them() {
    for unsupported in [
        "id='strict'\nfuture_profile_field=true\n[[bones]]\nbone_path='root'\n[bones.shape]\nkind='sphere'\nradius=0.2\n",
        "id='strict'\n[[bones]]\nbone_path='root'\nfuture_bone_field=true\n[bones.shape]\nkind='sphere'\nradius=0.2\n",
    ] {
        assert!(matches!(RagdollProfile::from_toml(unsupported), Err(super::RagdollProfileError::Parse(_))));
    }
}
