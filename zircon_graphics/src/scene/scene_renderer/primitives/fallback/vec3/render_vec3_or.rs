use zircon_math::{to_render_vec3, RenderVec3, Vec3};

pub(crate) fn render_vec3_or(value: Vec3, fallback: RenderVec3) -> RenderVec3 {
    to_render_vec3(value).unwrap_or(fallback)
}
