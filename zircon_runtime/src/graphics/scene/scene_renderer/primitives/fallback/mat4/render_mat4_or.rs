use crate::core::math::{to_render_mat4, Mat4, RenderMat4};

pub(crate) fn render_mat4_or(value: Mat4, fallback: RenderMat4) -> RenderMat4 {
    to_render_mat4(value).unwrap_or(fallback)
}
