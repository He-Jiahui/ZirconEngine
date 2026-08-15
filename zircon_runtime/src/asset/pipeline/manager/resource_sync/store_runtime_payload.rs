use crate::asset::{AssetId, ImportedAsset};
use crate::core::resource::{ResourceManager, ResourceMutationBatch, ResourceResult};

pub(in crate::asset::pipeline::manager) fn store_runtime_payload(
    resource_manager: &ResourceManager,
    id: AssetId,
    expected_revision: u64,
    imported: ImportedAsset,
) -> ResourceResult<()> {
    resource_manager.commit(ResourceMutationBatch::new().store_payload_erased(
        id,
        expected_revision,
        imported.into_resource_data(),
    ))?;
    Ok(())
}
