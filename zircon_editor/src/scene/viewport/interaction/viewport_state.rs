use zircon_runtime_interface::math::UVec2;

#[derive(Clone, Debug)]
pub struct ViewportState {
    pub size: UVec2,
}

impl ViewportState {
    pub fn new(size: UVec2) -> Self {
        Self {
            size: UVec2::new(size.x.max(1), size.y.max(1)),
        }
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new(UVec2::new(960, 540))
    }
}
