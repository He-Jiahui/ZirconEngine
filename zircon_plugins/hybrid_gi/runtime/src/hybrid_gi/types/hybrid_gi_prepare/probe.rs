#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridGiPrepareProbe {
    pub probe_id: u32,
    pub slot: u32,
    pub ray_budget: u32,
    pub irradiance_rgb: [u8; 3],
}
