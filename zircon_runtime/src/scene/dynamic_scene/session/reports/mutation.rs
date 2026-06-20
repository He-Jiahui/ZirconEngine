use super::super::RuntimeSessionMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotMutationPreviewReport {
    pub source_slot_id: String,
    pub destination_slot_id: Option<String>,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}
