#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiRuntimeTraceRegionSceneData {
    center_x_q: u32,
    center_y_q: u32,
    center_z_q: u32,
    radius_q: u32,
    coverage_q: u32,
    rt_lighting_rgb: [u8; 3],
}

impl HybridGiRuntimeTraceRegionSceneData {
    pub(in crate::hybrid_gi) fn new(
        center_x_q: u32,
        center_y_q: u32,
        center_z_q: u32,
        radius_q: u32,
        coverage_q: u32,
        rt_lighting_rgb: [u8; 3],
    ) -> Self {
        Self {
            center_x_q,
            center_y_q,
            center_z_q,
            radius_q,
            coverage_q,
            rt_lighting_rgb,
        }
    }

    pub(in crate::hybrid_gi) fn center_x_q(&self) -> u32 {
        self.center_x_q
    }

    pub(in crate::hybrid_gi) fn center_y_q(&self) -> u32 {
        self.center_y_q
    }

    pub(in crate::hybrid_gi) fn center_z_q(&self) -> u32 {
        self.center_z_q
    }

    pub(in crate::hybrid_gi) fn radius_q(&self) -> u32 {
        self.radius_q
    }

    pub(in crate::hybrid_gi) fn coverage_q(&self) -> u32 {
        self.coverage_q
    }

    pub(in crate::hybrid_gi) fn rt_lighting_rgb(&self) -> [u8; 3] {
        self.rt_lighting_rgb
    }
}
