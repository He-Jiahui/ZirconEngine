use zircon_runtime::core::math::Vec3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParticleRng {
    state: u64,
}

impl ParticleRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub(crate) fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    pub(crate) fn next_unit(&mut self) -> f32 {
        let value = self.next_u32();
        (value as f32) / (u32::MAX as f32)
    }

    pub(crate) fn next_signed(&mut self) -> f32 {
        self.next_unit() * 2.0 - 1.0
    }

    pub(crate) fn next_unit_vector(&mut self) -> Vec3 {
        let z = self.next_signed();
        let angle = self.next_unit() * std::f32::consts::TAU;
        let radius = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(radius * angle.cos(), z, radius * angle.sin())
    }
}
