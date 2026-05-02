use zircon_runtime_interface::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportRenderablePickCandidate {
    pub owner: u64,
    pub position: Vec3,
    pub radius_world: f32,
}
