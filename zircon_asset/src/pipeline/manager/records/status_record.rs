use zircon_resource::ResourceState;

use super::{metadata_import_state::metadata_import_state, AssetStatusRecord};
use crate::AssetMetadata;

pub(in crate::pipeline::manager) fn build_status_record(
    metadata: &AssetMetadata,
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
