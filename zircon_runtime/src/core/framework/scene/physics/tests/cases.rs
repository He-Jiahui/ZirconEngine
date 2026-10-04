use super::{
    PhysicsCombineRule, PhysicsJointConstraintMetadata, PhysicsJointDrive, PhysicsMaterialMetadata,
};

#[test]
fn material_metadata_roundtrips_without_a_simulation_contract() {
    let material = PhysicsMaterialMetadata {
        static_friction: 0.8,
        dynamic_friction: 0.6,
        restitution: 0.25,
        friction_combine: PhysicsCombineRule::Maximum,
        restitution_combine: PhysicsCombineRule::Multiply,
    };

    assert_eq!(
        serde_json::from_value::<PhysicsMaterialMetadata>(serde_json::to_value(&material).unwrap())
            .unwrap(),
        material
    );
}

#[test]
fn joint_constraint_metadata_roundtrips_sparse_axis_limits() {
    let default_document = toml::to_string_pretty(&PhysicsJointConstraintMetadata::default())
        .expect("default joint constraint should serialize as TOML");
    assert!(!default_document.contains("linear_limits"));
    assert!(!default_document.contains("angular_limits"));
    assert_eq!(
        toml::from_str::<PhysicsJointConstraintMetadata>(&default_document)
            .expect("default joint constraint TOML should deserialize"),
        PhysicsJointConstraintMetadata::default()
    );

    let constraint = PhysicsJointConstraintMetadata {
        linear_limits: [Some([-0.2, 0.2]), None, Some([0.0, 1.0])],
        angular_limits: [None, Some([-0.25, 0.25]), None],
        linear_drives: [
            PhysicsJointDrive {
                target_position: 0.1,
                stiffness: 12.0,
                damping: 2.0,
                max_force: 30.0,
                ..PhysicsJointDrive::default()
            },
            PhysicsJointDrive::default(),
            PhysicsJointDrive::default(),
        ],
        break_force: Some(120.0),
        ..PhysicsJointConstraintMetadata::default()
    };

    let document = toml::to_string_pretty(&constraint)
        .expect("sparse joint constraint should serialize as TOML");
    let loaded = toml::from_str::<PhysicsJointConstraintMetadata>(&document)
        .expect("sparse joint constraint TOML should deserialize");
    assert_eq!(loaded, constraint);

    let value = toml::from_str::<toml::Table>(&document)
        .expect("sparse joint constraint should be valid TOML");
    let linear_limits = value
        .get("linear_limits")
        .and_then(toml::Value::as_table)
        .expect("linear limits should serialize as an axis table");
    assert!(linear_limits.contains_key("x"));
    assert!(!linear_limits.contains_key("y"));
    assert!(linear_limits.contains_key("z"));

    let array_axis_limits = serde_json::json!({
        "linear_limits": [[-0.2, 0.2], null, [0.0, 1.0]],
        "angular_limits": [null, [-0.25, 0.25], null],
    });
    let array_constraint =
        serde_json::from_value::<PhysicsJointConstraintMetadata>(array_axis_limits).unwrap();
    assert_eq!(array_constraint.linear_limits, constraint.linear_limits);
    assert_eq!(array_constraint.angular_limits, constraint.angular_limits);
}

#[test]
fn joint_constraint_metadata_rejects_invalid_axis_documents() {
    let duplicate_axis = r#"{
        "linear_limits": {
            "x": [-0.2, 0.2],
            "x": [0.0, 1.0]
        }
    }"#;
    let duplicate_error = serde_json::from_str::<PhysicsJointConstraintMetadata>(duplicate_axis)
        .expect_err("duplicate axis entries must be rejected");
    assert!(duplicate_error.to_string().contains("duplicate field `x`"));

    let unknown_axis = r#"{
        "angular_limits": {
            "w": [-0.25, 0.25]
        }
    }"#;
    let unknown_error = serde_json::from_str::<PhysicsJointConstraintMetadata>(unknown_axis)
        .expect_err("unknown axis entries must be rejected");
    assert!(unknown_error.to_string().contains("unknown field `w`"));

    let too_many_axes = serde_json::json!({
        "linear_limits": [null, null, null, [0.0, 1.0]],
    });
    let length_error = serde_json::from_value::<PhysicsJointConstraintMetadata>(too_many_axes)
        .expect_err("axis arrays longer than three slots must be rejected");
    assert!(length_error.to_string().contains("invalid length 4"));
}
