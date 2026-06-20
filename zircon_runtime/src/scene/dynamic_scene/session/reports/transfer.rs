use std::path::PathBuf;

use super::super::RuntimeSessionMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotImportPreviewReport {
    pub source_slot_id: String,
    pub destination_slot_id: String,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotExportPreviewReport {
    pub source_slot_id: String,
    pub target_path: Option<PathBuf>,
    pub will_replace_target: bool,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}
