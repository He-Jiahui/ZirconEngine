use zircon_runtime::core::math::{Transform, Vec3};

#[derive(Clone, Copy, Debug)]
pub(crate) struct HandleBasis {
    pub(crate) origin: Transform,
    pub(crate) x: Vec3,
    pub(crate) y: Vec3,
    pub(crate) z: Vec3,
    pub(crate) extent: f32,
}
