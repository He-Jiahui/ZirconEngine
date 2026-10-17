//! 在运行时值进入渲染数据前检查有限性和标量范围；向量、矩阵逐分量转换，空间语义保持不变。

use super::{
    Mat4, Quat, Real, RenderMat4, RenderScalar, RenderVec2, RenderVec3, RenderVec4, UVec2, Vec2,
    Vec3, Vec4,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum RenderNarrowingError {
    #[error("runtime scalar must be finite before render narrowing")]
    NonFiniteSource,
    #[error("runtime scalar {value} is outside the render scalar range [{minimum}, {maximum}]")]
    OutsideRenderRange {
        value: Real,
        minimum: Real,
        maximum: Real,
    },
    #[error("render scalar result must remain finite after narrowing")]
    NonFiniteResult,
}

/// 显式运行时到渲染标量转换的记录，误差为源值与渲染值之差的绝对值。
/// 派生的 Deserialize 只恢复字段数据，不重新执行转换或验证误差关系。
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RenderNarrowingReceipt {
    source: Real,
    rendered: RenderScalar,
    absolute_error: Real,
}

impl RenderNarrowingReceipt {
    pub const fn source(self) -> Real {
        self.source
    }

    pub const fn rendered(self) -> RenderScalar {
        self.rendered
    }

    pub const fn absolute_error(self) -> Real {
        self.absolute_error
    }

    pub const fn is_exact(self) -> bool {
        self.absolute_error == 0.0
    }
}

pub fn is_finite_scalar(value: Real) -> bool {
    value.is_finite()
}

pub fn is_finite_vec2(value: Vec2) -> bool {
    value.is_finite()
}

pub fn is_finite_vec3(value: Vec3) -> bool {
    value.is_finite()
}

pub fn is_finite_vec4(value: Vec4) -> bool {
    value.is_finite()
}

pub fn is_finite_quat(value: Quat) -> bool {
    value.is_finite()
}

pub fn is_finite_mat4(value: Mat4) -> bool {
    value.is_finite()
}

/// 尝试窄化单个标量；失败时丢弃错误详情，需要区分失败原因时使用 try_to_render_scalar。
pub fn to_render_scalar(value: Real) -> Option<RenderScalar> {
    try_to_render_scalar(value)
        .ok()
        .map(RenderNarrowingReceipt::rendered)
}

/// 校验源值有限且在渲染标量范围内，成功时返回转换值和绝对误差记录。
pub fn try_to_render_scalar(value: Real) -> Result<RenderNarrowingReceipt, RenderNarrowingError> {
    if !is_finite_scalar(value) {
        return Err(RenderNarrowingError::NonFiniteSource);
    }

    let minimum = RenderScalar::MIN as Real;
    let maximum = RenderScalar::MAX as Real;
    if value < minimum || value > maximum {
        return Err(RenderNarrowingError::OutsideRenderRange {
            value,
            minimum,
            maximum,
        });
    }

    let rendered = value as RenderScalar;
    if !rendered.is_finite() {
        return Err(RenderNarrowingError::NonFiniteResult);
    }
    let absolute_error = (value - rendered as Real).abs();
    Ok(RenderNarrowingReceipt {
        source: value,
        rendered,
        absolute_error,
    })
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

/// 按原列数组顺序检查并转换 16 个分量；任一分量失败即返回 None，不验证矩阵是否可逆或属于某个空间。
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

/// 将视口宽、高分别限制为至少 1，避免后续按零尺寸计算。
pub fn clamp_viewport_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}
