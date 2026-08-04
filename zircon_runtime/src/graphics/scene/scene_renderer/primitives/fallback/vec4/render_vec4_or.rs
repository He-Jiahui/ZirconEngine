use crate::core::math::{RenderVec4, Vec4, to_render_vec4};

pub(crate) fn render_vec4_or(value: Vec4, fallback: RenderVec4) -> RenderVec4 {
    to_render_vec4(value).unwrap_or(fallback)
}
