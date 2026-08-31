//! Stable math projection and versioned schema DTOs for runtime products.

mod schema;

#[cfg(test)]
mod tests;

pub use schema::{
    CoordinateSchema, PrecisionProfile, UnitSchema, ZIRCON_COORDINATE_SCHEMA,
    ZIRCON_PRECISION_PROFILE, ZIRCON_UNIT_SCHEMA,
};
pub use zr_math::{
    affine_inverse, clamp_viewport_size, compose_trs, is_finite_mat4, is_finite_quat,
    is_finite_scalar, is_finite_vec2, is_finite_vec3, is_finite_vec4, perspective, to_render_mat4,
    to_render_scalar, to_render_vec2, to_render_vec3, to_render_vec4, transform_to_mat4,
    try_affine_inverse, try_perspective, try_to_render_scalar, view_matrix, AffineInverseError,
    AngleUnit, Axis3, AxisDirection, ClipDepthRange, CoordinateHandedness, DepthDirection,
    EulerRot, FrontFaceWinding, LengthUnit, LookAtError, Mat4, MatrixConvention, Normal3,
    NumericError, NumericPolicy, NumericPolicyError, NumericPolicyField, NumericPolicyThresholds,
    NumericValue, PerspectiveError, Position3, Quat, Real, RenderMat4, RenderNarrowingError,
    RenderNarrowingReceipt, RenderScalar, RenderVec2, RenderVec3, RenderVec4, ScalarPrecision,
    SpaceKind, SpatialError, SpatialValue, TimeUnit, Transform, UVec2, UnitDirection3,
    UnitQuaternion, ValidatedPerspective, ValidatedTransform, Vec2, Vec3, Vec4, Vector3,
};
