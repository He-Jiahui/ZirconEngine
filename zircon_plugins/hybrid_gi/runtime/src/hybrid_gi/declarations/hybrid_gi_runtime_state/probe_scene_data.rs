#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiRuntimeProbeSceneData {
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
}

impl HybridGiRuntimeProbeSceneData {
    pub(in crate::hybrid_gi) fn new(
        position_x_q: u32,
        position_y_q: u32,
        position_z_q: u32,
        radius_q: u32,
    ) -> Self {
        Self {
            position_x_q,
            position_y_q,
            position_z_q,
            radius_q,
        }
    }

    pub(in crate::hybrid_gi) fn position_x_q(&self) -> u32 {
        self.position_x_q
    }

    pub(in crate::hybrid_gi) fn position_y_q(&self) -> u32 {
        self.position_y_q
    }

    pub(in crate::hybrid_gi) fn position_z_q(&self) -> u32 {
        self.position_z_q
    }

    pub(in crate::hybrid_gi) fn radius_q(&self) -> u32 {
        self.radius_q
    }
}
