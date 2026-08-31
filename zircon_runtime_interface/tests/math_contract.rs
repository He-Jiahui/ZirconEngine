use zircon_runtime_interface::math::{
    try_affine_inverse, try_perspective, try_to_render_scalar, AffineInverseError, Axis3,
    AxisDirection, ClipDepthRange, CoordinateHandedness, DepthDirection, FrontFaceWinding, Mat4,
    MatrixConvention, NumericError, NumericPolicy, NumericValue, PerspectiveError, Position3, Quat,
    ScalarPrecision, SpaceKind, SpatialError, TimeUnit, Transform, UnitDirection3,
    ValidatedPerspective, ValidatedTransform, Vec3, Vector3, ZIRCON_COORDINATE_SCHEMA,
    ZIRCON_PRECISION_PROFILE,
};

#[test]
fn public_coordinate_schema_preserves_zircon_space_convention() {
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.handedness,
        CoordinateHandedness::Right
    );
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.up,
        AxisDirection::Positive(Axis3::Y)
    );
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.forward,
        AxisDirection::Negative(Axis3::Z)
    );
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.matrix_convention,
        MatrixConvention::ColumnVectorColumnMajor
    );
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.clip_depth_range,
        ClipDepthRange::ZeroToOne
    );
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.depth_direction,
        DepthDirection::NearToFar
    );
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.canonical_front_face,
        FrontFaceWinding::CounterClockwise
    );
    assert_eq!(
        ZIRCON_PRECISION_PROFILE.runtime_scalar,
        ScalarPrecision::F32
    );
    assert_eq!(ZIRCON_PRECISION_PROFILE.render_scalar, ScalarPrecision::F32);
    assert_eq!(
        zircon_runtime_interface::math::ZIRCON_UNIT_SCHEMA.time,
        TimeUnit::Second
    );
}

#[test]
fn public_transform_validation_rejects_degenerate_inputs() {
    assert!(matches!(
        UnitDirection3::try_new(Vec3::ZERO, NumericPolicy::STRICT),
        Err(NumericError::NormTooSmall {
            value: NumericValue::Direction,
            ..
        })
    ));

    let invalid_scale = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::new(1.0, 0.0, 1.0),
    };
    assert!(matches!(
        ValidatedTransform::try_new(invalid_scale, NumericPolicy::STRICT),
        Err(NumericError::ScaleTooSmall { axis: Axis3::Y, .. })
    ));
}

#[test]
fn public_spatial_values_keep_space_explicit() {
    let world_position =
        Position3::try_new(Vec3::ZERO, SpaceKind::World).expect("finite position is valid");
    let world_delta = Vector3::try_new(Vec3::X, SpaceKind::World).expect("finite vector is valid");
    assert_eq!(
        world_position
            .checked_add(world_delta)
            .expect("matching spaces compose")
            .value(),
        Vec3::X
    );

    let local_delta = Vector3::try_new(Vec3::X, SpaceKind::Local).expect("finite vector is valid");
    assert!(matches!(
        world_position.checked_add(local_delta),
        Err(SpatialError::SpaceMismatch { .. })
    ));
}

#[test]
fn public_checked_render_narrowing_returns_conversion_evidence() {
    let receipt = try_to_render_scalar(12.5).expect("current f32 scalar is renderable");

    assert_eq!(receipt.source(), 12.5);
    assert_eq!(receipt.rendered(), 12.5);
    assert!(receipt.is_exact());
}

#[test]
fn public_fallible_math_apis_reject_invalid_geometry() {
    assert!(matches!(
        try_perspective(0.0, 1.0, 0.1, 100.0),
        Err(PerspectiveError::FieldOfViewOutOfRange)
    ));
    assert!(matches!(
        try_affine_inverse(
            Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0)),
            NumericPolicy::STRICT,
        ),
        Err(AffineInverseError::DeterminantTooSmall { .. })
    ));
    assert!(matches!(
        try_affine_inverse(
            Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0),
            NumericPolicy::STRICT,
        ),
        Err(AffineInverseError::NonAffineInput)
    ));
    assert!(
        Transform::try_looking_at(Vec3::ZERO, -Vec3::Z, -Vec3::Z, NumericPolicy::STRICT,).is_err()
    );
}

#[test]
fn public_validated_perspective_is_bound_to_the_current_depth_contract() {
    let projection =
        ValidatedPerspective::new(1.0, 1.0, 0.1, 100.0).expect("valid finite projection inputs");

    assert_eq!(projection.depth_direction(), DepthDirection::NearToFar);
    assert_eq!(
        projection.matrix(),
        Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0)
    );
}
