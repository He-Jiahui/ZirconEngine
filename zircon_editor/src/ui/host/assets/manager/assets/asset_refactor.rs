use zircon_runtime::asset::AssetUuid;
use zircon_runtime::core::CoreError;

use crate::core::asset::{AssetDeletePreflight, AssetSourceWritePolicy};
use crate::ui::host::module::EDITOR_ASSET_MANAGER_NAME;

use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    /// Projects a delete admission from the active runtime registry without creating an editor
    /// copy of its reference graph.
    pub fn asset_delete_preflight(
        &self,
        asset_uuid: AssetUuid,
        write_policy: AssetSourceWritePolicy,
    ) -> Result<AssetDeletePreflight, CoreError> {
        let state = self.read_state_recovering_poison();
        let project = state.project.as_ref().ok_or_else(|| {
            CoreError::Initialization(
                EDITOR_ASSET_MANAGER_NAME.to_string(),
                "asset delete preflight requires an active runtime project".to_string(),
            )
        })?;
        Ok(AssetDeletePreflight::evaluate(
            project.asset_registry(),
            asset_uuid,
            write_policy,
        ))
    }
}
