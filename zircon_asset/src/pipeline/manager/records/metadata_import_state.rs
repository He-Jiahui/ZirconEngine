use zircon_resource::{ResourceRecord, ResourceState};

pub(super) fn metadata_import_state(metadata: &ResourceRecord) -> ResourceState {
    if metadata.artifact_locator().is_some() {
        ResourceState::Ready
    } else {
        ResourceState::Pending
    }
}
