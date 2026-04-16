use crate::ResourceRecord;

pub(super) fn next_ready_revision(previous: &ResourceRecord, next: &ResourceRecord) -> u64 {
    if ready_record_changed(previous, next) {
        previous.revision + 1
    } else {
        previous.revision
    }
}

fn ready_record_changed(previous: &ResourceRecord, next: &ResourceRecord) -> bool {
    previous.kind != next.kind
        || previous.primary_locator != next.primary_locator
        || previous.artifact_locator != next.artifact_locator
        || previous.source_hash != next.source_hash
        || previous.importer_version != next.importer_version
        || previous.config_hash != next.config_hash
        || previous.dependency_ids != next.dependency_ids
}
