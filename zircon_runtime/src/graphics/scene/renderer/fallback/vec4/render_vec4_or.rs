use crate::core::math::{to_render_vec4, RenderVec4, Vec4};

pub(crate) fn render_vec4_or(value: Vec4, fallback: RenderVec4) -> RenderVec4 {
    to_render_vec4(value).unwrap_or(fallback)
}
