#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiPrepareProbe {
    pub(crate) probe_id: u32,
    pub(crate) slot: u32,
    pub(crate) ray_budget: u32,
    pub(crate) irradiance_rgb: [u8; 3],
}
