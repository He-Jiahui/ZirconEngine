use zircon_runtime::core::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub struct HybridGiPrepareCardCaptureRequest {
    pub card_id: u32,
    pub page_id: u32,
    pub atlas_slot_id: u32,
    pub capture_slot_id: u32,
    pub bounds_center: Vec3,
    pub bounds_radius: f32,
}
