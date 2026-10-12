#[derive(Clone, Debug, PartialEq, Eq)]
/// 已分配槽的探针输入；来源掩码与 Q8 动态权重随 RGB8 一起传给 GPU 准备阶段。
pub struct HybridGiPrepareProbe {
    pub probe_id: u32,
    pub slot: u32,
    pub stable_instance_key: u64,
    pub source_mask: u32,
    pub dynamic_weight_q8: u8,
    pub ray_budget: u32,
    pub irradiance_rgb: [u8; 3],
}
