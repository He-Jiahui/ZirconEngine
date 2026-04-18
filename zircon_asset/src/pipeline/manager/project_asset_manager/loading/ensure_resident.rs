use zircon_core::CoreError;

use super::super::super::builtins::builtin_resources;
use super::super::super::errors::{asset_error, asset_error_message};
use super::super::super::resource_sync::store_runtime_payload;
use super::super::ProjectAssetManager;
use crate::{AssetId, AssetUri, AssetUriScheme};

impl ProjectAssetManager {
    pub(in crate::pipeline::manager::project_asset_manager::loading) fn ensure_resident(
        &self,
        id: AssetId,
    ) -> Result<(), CoreError> {
        if self.resource_manager().get_untyped(id).is_some() {
            return Ok(());
        }

        let metadata = self
            .resource_manager()
            .registry()
            .get(id)
            .cloned()
            .ok_or_else(|| {
                asset_error_message(format!("missing resource record for asset id {id}"))
            })?;
        let imported = match metadata.primary_locator.scheme() {
            AssetUriScheme::Builtin => builtin_resources()
                .into_iter()
                .find_map(|(locator_text, asset)| {
                    let locator = AssetUri::parse(locator_text).ok()?;
                    (locator == metadata.primary_locator).then_some(asset)
                })
                .ok_or_else(|| {
                    asset_error_message(format!(
                        "missing builtin runtime payload for {}",
                        metadata.primary_locator
                    ))
                })?,
            AssetUriScheme::Res | AssetUriScheme::Library => {
                let project = self.project_read();
                let project = project
                    .as_ref()
                    .ok_or_else(|| asset_error_message("no project is currently open"))?;
                project.load_artifact_by_id(id).map_err(asset_error)?
            }
            AssetUriScheme::Memory => {
                return Err(asset_error_message(format!(
                    "memory resource {id} cannot be restored by ProjectAssetManager"
                )));
            }
        };
        store_runtime_payload(&self.resource_manager, id, imported);
        Ok(())
    }
}
