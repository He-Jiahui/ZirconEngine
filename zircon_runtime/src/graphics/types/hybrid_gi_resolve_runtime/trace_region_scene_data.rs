#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiResolveTraceRegionSceneData {
    center_x_q: u32,
    center_y_q: u32,
    center_z_q: u32,
    radius_q: u32,
    coverage_q: u32,
    rt_lighting_rgb: [u8; 3],
}

impl HybridGiResolveTraceRegionSceneData {
    pub(crate) fn new(
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

    pub(crate) fn center_x_q(&self) -> u32 {
        self.center_x_q
    }

    pub(crate) fn center_y_q(&self) -> u32 {
        self.center_y_q
    }

    pub(crate) fn center_z_q(&self) -> u32 {
        self.center_z_q
    }

    pub(crate) fn radius_q(&self) -> u32 {
        self.radius_q
    }

    pub(crate) fn coverage_q(&self) -> u32 {
        self.coverage_q
    }

    pub(crate) fn rt_lighting_rgb(&self) -> [u8; 3] {
        self.rt_lighting_rgb
    }
}
