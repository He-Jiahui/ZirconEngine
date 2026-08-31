use crate::{ResourceRecord, ResourceRegistryError, ResourceResult};

pub(super) fn next_ready_revision(
    previous: &ResourceRecord,
    next: &ResourceRecord,
) -> ResourceResult<u64> {
    if previous.revision == 0 {
        Ok(1)
    } else if ready_record_changed(previous, next) {
        previous
            .revision
            .checked_add(1)
            .ok_or_else(|| ResourceRegistryError::RevisionExhausted {
                id: previous.id.to_string(),
                current_revision: previous.revision,
            })
    } else {
        Ok(previous.revision)
    }
}

fn ready_record_changed(previous: &ResourceRecord, next: &ResourceRecord) -> bool {
    previous.kind != next.kind
        || previous.primary_locator != next.primary_locator
        || previous.artifact_locator != next.artifact_locator
        || previous.source_hash != next.source_hash
        || previous.importer_id != next.importer_id
        || previous.importer_version != next.importer_version
        || previous.config_hash != next.config_hash
        || previous.dependency_ids != next.dependency_ids
}
