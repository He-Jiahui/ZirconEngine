//! Shared runtime math types and the render-precision seam.

mod precision;
mod transform;

pub use glam::{EulerRot, UVec2};
pub use precision::{
    is_finite_mat4, is_finite_quat, is_finite_scalar, is_finite_vec2, is_finite_vec3,
    is_finite_vec4, to_render_mat4, to_render_scalar, to_render_vec2, to_render_vec3,
    to_render_vec4, Mat4, Quat, Real, RenderMat4, RenderScalar, RenderVec2, RenderVec3, RenderVec4,
    Vec2, Vec3, Vec4,
};
pub use transform::{
    affine_inverse, clamp_viewport_size, compose_trs, perspective, transform_to_mat4, view_matrix,
    Transform,
};
