use zircon_math::{Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub(super) struct ParsedObjVertex {
    pub(super) position: Vec3,
    pub(super) uv: Vec2,
    pub(super) normal: Vec3,
    pub(super) needs_generated_normal: bool,
}
