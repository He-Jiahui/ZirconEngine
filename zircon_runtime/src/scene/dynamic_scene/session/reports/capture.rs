use super::super::RuntimeSessionMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotCapturePreviewReport {
    pub slot_id: String,
    pub will_replace_existing: bool,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}
