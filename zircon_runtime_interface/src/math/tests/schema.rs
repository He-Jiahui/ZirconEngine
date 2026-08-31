use crate::math::{
    Axis3, AxisDirection, ClipDepthRange, CoordinateHandedness, DepthDirection, FrontFaceWinding,
    LengthUnit, MatrixConvention, PrecisionProfile, ScalarPrecision, SpaceKind, TimeUnit,
    ZIRCON_COORDINATE_SCHEMA, ZIRCON_PRECISION_PROFILE, ZIRCON_UNIT_SCHEMA,
};

#[test]
fn zircon_coordinate_schema_is_right_handed_y_up_and_negative_z_forward() {
    assert_eq!(
        ZIRCON_COORDINATE_SCHEMA.schema_id.as_str(),
        "zircon.coordinate"
    );
    assert_eq!(ZIRCON_COORDINATE_SCHEMA.version, 1);
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
    assert_eq!(ZIRCON_UNIT_SCHEMA.schema_id.as_str(), "zircon.units");
    assert_eq!(ZIRCON_UNIT_SCHEMA.length, LengthUnit::Meter);
    assert_eq!(ZIRCON_UNIT_SCHEMA.time, TimeUnit::Second);
    assert!(SpaceKind::ViewRelative.is_render_space());
    assert!(!SpaceKind::World.is_render_space());
}

#[test]
fn current_precision_profile_remains_f32_backed() {
    assert_eq!(
        ZIRCON_PRECISION_PROFILE.schema_id.as_str(),
        "zircon.precision"
    );
    assert_eq!(ZIRCON_PRECISION_PROFILE.version, 1);
    assert_eq!(
        ZIRCON_PRECISION_PROFILE.runtime_scalar,
        ScalarPrecision::F32
    );
    assert_eq!(ZIRCON_PRECISION_PROFILE.render_scalar, ScalarPrecision::F32);
    assert_eq!(PrecisionProfile::CURRENT, ZIRCON_PRECISION_PROFILE);
    assert_eq!(
        PrecisionProfile::CURRENT.cpu_scalar_bytes(),
        std::mem::size_of::<crate::math::Real>()
    );
    assert_eq!(
        PrecisionProfile::CURRENT.render_scalar_bytes(),
        std::mem::size_of::<crate::math::RenderScalar>()
    );
}
