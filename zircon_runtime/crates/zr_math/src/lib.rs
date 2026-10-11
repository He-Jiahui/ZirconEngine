#![forbid(unsafe_code)]

//! Canonical math primitives, policies, and checked operations for Zircon.

//! 提供共享数学值、数值阈值与受检运算；版本化的坐标、单位和精度契约由接口层定义。
//! 当前运行时和渲染别名都使用 f32，各产品通过自己的数学公开入口使用这些定义。

mod conventions;
mod fallible;
mod numeric_policy;
mod render_conversion;
mod space;
mod transform;

#[cfg(test)]
mod tests;

pub use conventions::{
    AngleUnit, Axis3, AxisDirection, ClipDepthRange, CoordinateHandedness, DepthDirection,
    FrontFaceWinding, LengthUnit, MatrixConvention, ScalarPrecision, SpaceKind, TimeUnit,
};
pub use fallible::{
    perspective, try_affine_inverse, try_perspective, AffineInverseError, PerspectiveError,
    ValidatedPerspective,
};
pub use numeric_policy::{
    NumericError, NumericPolicy, NumericPolicyError, NumericPolicyField, NumericPolicyThresholds,
    NumericValue,
};
pub use render_conversion::{
    clamp_viewport_size, is_finite_mat4, is_finite_quat, is_finite_scalar, is_finite_vec2,
    is_finite_vec3, is_finite_vec4, to_render_mat4, to_render_scalar, to_render_vec2,
    to_render_vec3, to_render_vec4, try_to_render_scalar, RenderNarrowingError,
    RenderNarrowingReceipt,
};
pub use space::{Normal3, Position3, SpatialError, SpatialValue, Vector3};
pub use transform::{
    affine_inverse, compose_trs, transform_to_mat4, view_matrix, LookAtError, Transform,
    UnitDirection3, UnitQuaternion, ValidatedTransform,
};

pub use glam::{EulerRot, UVec2};

/// 当前运行时标量采用 f32；精度枚举描述边界元数据，不会改变此别名的存储类型。
pub type Real = f32;
pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;
pub type Quat = glam::Quat;
pub type Mat4 = glam::Mat4;

pub type RenderScalar = f32;
pub type RenderVec2 = glam::Vec2;
pub type RenderVec3 = glam::Vec3;
pub type RenderVec4 = glam::Vec4;
pub type RenderMat4 = glam::Mat4;
