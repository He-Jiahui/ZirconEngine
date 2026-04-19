use super::aliases::{
    Mat4, Real, RenderMat4, RenderScalar, RenderVec2, RenderVec3, RenderVec4, Vec2, Vec3, Vec4,
};
use super::finite::is_finite_scalar;

pub fn to_render_scalar(value: Real) -> Option<RenderScalar> {
    is_finite_scalar(value).then(|| value as RenderScalar)
}

pub fn to_render_vec2(value: Vec2) -> Option<RenderVec2> {
    Some(RenderVec2::new(
        to_render_scalar(value.x)?,
        to_render_scalar(value.y)?,
    ))
}

pub fn to_render_vec3(value: Vec3) -> Option<RenderVec3> {
    Some(RenderVec3::new(
        to_render_scalar(value.x)?,
        to_render_scalar(value.y)?,
        to_render_scalar(value.z)?,
    ))
}

pub fn to_render_vec4(value: Vec4) -> Option<RenderVec4> {
    Some(RenderVec4::new(
        to_render_scalar(value.x)?,
        to_render_scalar(value.y)?,
        to_render_scalar(value.z)?,
        to_render_scalar(value.w)?,
    ))
}

pub fn to_render_mat4(value: Mat4) -> Option<RenderMat4> {
    let cols = value.to_cols_array();
    Some(RenderMat4::from_cols_array(&[
        to_render_scalar(cols[0])?,
        to_render_scalar(cols[1])?,
        to_render_scalar(cols[2])?,
        to_render_scalar(cols[3])?,
        to_render_scalar(cols[4])?,
        to_render_scalar(cols[5])?,
        to_render_scalar(cols[6])?,
        to_render_scalar(cols[7])?,
        to_render_scalar(cols[8])?,
        to_render_scalar(cols[9])?,
        to_render_scalar(cols[10])?,
        to_render_scalar(cols[11])?,
        to_render_scalar(cols[12])?,
        to_render_scalar(cols[13])?,
        to_render_scalar(cols[14])?,
        to_render_scalar(cols[15])?,
    ]))
}
