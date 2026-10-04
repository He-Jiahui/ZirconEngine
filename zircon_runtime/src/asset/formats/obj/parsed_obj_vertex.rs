use crate::core::math::{Vec2, Vec3};

/// 暂存法线来源，供解码末尾只为缺失法线的顶点生成平滑结果。
#[derive(Clone, Copy, Debug)]
pub(super) struct ParsedObjVertex {
    pub(super) position: Vec3,
    pub(super) uv: Vec2,
    pub(super) normal: Vec3,
    pub(super) needs_generated_normal: bool,
}
