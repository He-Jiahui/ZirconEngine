use zircon_runtime::core::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
/// 一次脏页采集请求：卡 ID 是本次呈现中的所有者，跨帧身份另由稳定实例键映射。
pub struct HybridGiPrepareCardCaptureRequest {
    pub card_id: u32,
    pub page_id: u32,
    pub atlas_slot_id: u32,
    pub capture_slot_id: u32,
    pub bounds_center: Vec3,
    pub bounds_radius: f32,
}
