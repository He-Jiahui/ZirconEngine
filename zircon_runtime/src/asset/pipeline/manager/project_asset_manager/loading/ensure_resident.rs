use crate::core::resource::{ResourceScheme, ResourceState};
use crate::core::CoreError;

use super::super::super::builtins::builtin_resources;
use super::super::super::errors::{asset_error, asset_error_message};
use super::super::super::resource_sync::store_runtime_payload;
use super::super::ProjectAssetManager;
use crate::asset::{AssetId, AssetUri};

impl ProjectAssetManager {
    pub(crate) fn ensure_resident(&self, id: AssetId) -> Result<(), CoreError> {
        let _residency = self.lock_residency(id);
        if self.resource_manager().get_untyped(id).is_some() {
            return Ok(());
        }

        let (metadata, prepared_project_read, project_generation) = {
            let _generation = self.project_generation_read();
            let metadata = self
                .resource_manager()
                .registry()
                .get(id)
                .cloned()
                .ok_or_else(|| {
                    asset_error_message(format!("missing resource record for asset id {id}"))
                })?;
            if metadata.state != ResourceState::Ready {
                return Err(asset_error_message(format!(
                    "asset {id} is not ready for residency: {:?}",
                    metadata.state
                )));
            }
            let (prepared_project_read, project_generation) =
                match metadata.primary_locator.scheme() {
                    ResourceScheme::Res | ResourceScheme::Library | ResourceScheme::Package => {
                        let project = self.project_read();
                        let project = project
                            .as_ref()
                            .ok_or_else(|| asset_error_message("no project is currently open"))?;
                        (
                            Some(
                                project
                                    .prepare_artifact_read_by_id(id)
                                    .map_err(asset_error)?,
                            ),
                            Some(project.catalog_input_generation().sequence()),
                        )
                    }
                    _ => (None, None),
                };
            (metadata, prepared_project_read, project_generation)
        };
        let imported = match metadata.primary_locator.scheme() {
            ResourceScheme::Builtin => builtin_resources()
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
            ResourceScheme::Res | ResourceScheme::Library | ResourceScheme::Package => {
                prepared_project_read
                    .ok_or_else(|| {
                        asset_error_message(format!(
                            "asset {id} is missing its prepared project artifact read"
                        ))
                    })?
                    .read()
                    .map_err(asset_error)?
            }
            ResourceScheme::Memory => {
                return Err(asset_error_message(format!(
                    "memory resource {id} cannot be restored by ProjectAssetManager"
                )));
            }
        };
        if let Some(expected_generation) = project_generation {
            let _generation = self.project_generation_read();
            let current_record = self.resource_manager().registry().get(id).cloned();
            let current_project_generation = self
                .project_read()
                .as_ref()
                .map(|project| project.catalog_input_generation().sequence());
            if current_record.as_ref() != Some(&metadata)
                || current_project_generation != Some(expected_generation)
            {
                return Err(asset_error_message(format!(
                    "asset {id} residency preparation was superseded by a newer project generation"
                )));
            }
            store_runtime_payload(&self.resource_manager, id, metadata.revision, imported)
                .map_err(asset_error)?;
            return Ok(());
        }
        store_runtime_payload(&self.resource_manager, id, metadata.revision, imported)
            .map_err(asset_error)?;
        Ok(())
    }
}
