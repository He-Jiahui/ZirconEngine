use zircon_runtime::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendSpaceWeights2([(u32, Real); 2]);

impl BlendSpaceWeights2 {
    pub(super) const fn new(pairs: [(u32, Real); 2]) -> Self {
        Self(pairs)
    }

    pub const fn as_pairs(self) -> [(u32, Real); 2] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendSpaceWeights3([(u32, Real); 3]);

impl BlendSpaceWeights3 {
    pub(super) const fn new(pairs: [(u32, Real); 3]) -> Self {
        Self(pairs)
    }

    pub const fn as_pairs(self) -> [(u32, Real); 3] {
        self.0
    }

    pub fn weight_sum(self) -> Real {
        self.0.iter().map(|(_, weight)| weight).sum()
    }
}
