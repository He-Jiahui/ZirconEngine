use zircon_resource::{ResourceRecord, ResourceState};

use super::{metadata_import_state::metadata_import_state, AssetStatusRecord};

pub(in crate::pipeline::manager) fn build_status_record(
    metadata: &ResourceRecord,
) -> AssetStatusRecord {
    AssetStatusRecord {
        id: metadata.id().to_string(),
        uri: metadata.primary_locator().to_string(),
        kind: metadata.kind,
        artifact_uri: metadata.artifact_locator().map(ToString::to_string),
        imported: metadata_import_state(metadata) == ResourceState::Ready,
        source_hash: metadata.source_hash.clone(),
        importer_version: metadata.importer_version,
        config_hash: metadata.config_hash.clone(),
    }
}
