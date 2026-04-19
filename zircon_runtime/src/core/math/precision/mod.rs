mod aliases;
mod finite;
mod render;

pub use aliases::{
    Mat4, Quat, Real, RenderMat4, RenderScalar, RenderVec2, RenderVec3, RenderVec4, Vec2, Vec3,
    Vec4,
};
pub use finite::{
    is_finite_mat4, is_finite_quat, is_finite_scalar, is_finite_vec2, is_finite_vec3,
    is_finite_vec4,
};
pub use render::{
    to_render_mat4, to_render_scalar, to_render_vec2, to_render_vec3, to_render_vec4,
};
