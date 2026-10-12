use crate::asset::ImportedAsset;
use crate::core::resource::{ResourceMutationBatch, ResourceRecord};

pub(in crate::asset::pipeline::manager) fn register_project_resource(
    batch: ResourceMutationBatch,
    metadata: ResourceRecord,
    imported: ImportedAsset,
) -> ResourceMutationBatch {
    batch.upsert_imported_erased(metadata, imported.into_resource_data())
}
