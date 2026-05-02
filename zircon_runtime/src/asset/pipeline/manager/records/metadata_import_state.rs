use crate::core::resource::{ResourceRecord, ResourceState};

pub(super) fn metadata_import_state(metadata: &ResourceRecord) -> ResourceState {
    metadata.state
}
