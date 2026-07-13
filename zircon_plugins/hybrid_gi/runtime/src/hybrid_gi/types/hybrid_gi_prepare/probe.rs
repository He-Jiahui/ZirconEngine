#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridGiPrepareProbe {
    pub probe_id: u32,
    pub slot: u32,
    pub stable_instance_key: u64,
    pub source_mask: u32,
    pub dynamic_weight_q8: u8,
    pub ray_budget: u32,
    pub irradiance_rgb: [u8; 3],
}
