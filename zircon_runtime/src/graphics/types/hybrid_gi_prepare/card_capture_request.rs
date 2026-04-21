use crate::core::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HybridGiPrepareCardCaptureRequest {
    pub(crate) card_id: u32,
    pub(crate) page_id: u32,
    pub(crate) atlas_slot_id: u32,
    pub(crate) capture_slot_id: u32,
    pub(crate) bounds_center: Vec3,
    pub(crate) bounds_radius: f32,
}
