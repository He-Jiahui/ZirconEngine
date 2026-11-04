use std::collections::HashSet;

use zircon_runtime::asset::mutation::AssetMutationRelocationPreflight;
use zircon_runtime::asset::project::ProjectActivationIdentity;
use zircon_runtime::asset::{AssetUri, AssetUuid};
use zircon_runtime::core::resource::ResourceScheme;
use zircon_runtime::core::CoreError;

use crate::core::asset::{EditorAssetFolderOperation, EditorAssetImportState};
use crate::ui::host::module::EDITOR_ASSET_MANAGER_NAME;

use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub(super) fn project_mutation_activation(
        &self,
    ) -> Result<ProjectActivationIdentity, CoreError> {
        self.read_state_recovering_poison()
            .project
            .as_ref()
            .map(|project| project.activation_identity().clone())
            .ok_or_else(|| mutation_error("asset operation requires an active Editor project"))
    }

    pub(super) fn require_clean_folder_mutation(
        &self,
        expected: &ProjectActivationIdentity,
        operation: EditorAssetFolderOperation,
        source: &AssetUri,
    ) -> Result<(), CoreError> {
        let state = self.read_state_recovering_poison();
        if state
            .project
            .as_ref()
            .is_none_or(|project| project.activation_identity() != expected)
        {
            return Err(mutation_error(
                "folder operation belongs to an inactive Editor project",
            ));
        }
        if operation == EditorAssetFolderOperation::Create {
            return Ok(());
        }
        let index = state
            .asset_index
            .as_ref()
            .ok_or_else(|| mutation_error("folder operation requires the current asset index"))?;
        let index = index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if index.project_activation_identity() != Some(expected) {
            return Err(mutation_error(
                "folder operation asset index belongs to an inactive project",
            ));
        }
        let registry = index.runtime_registry();
        let mut affected = HashSet::<AssetUuid>::new();
        for entry in registry
            .entries_iter()
            .filter(|entry| folder_contains(source, entry.path()))
        {
            affected.insert(entry.uuid());
            if operation == EditorAssetFolderOperation::Rename {
                affected.extend(registry.get_referencers_by_uuid(entry.uuid()));
            }
        }
        for uuid in affected {
            if let Some(row) = index.row_by_uuid(uuid) {
                if row.import_state() == EditorAssetImportState::Importing {
                    return Err(mutation_error(format!(
                        "folder operation cannot change {} while an import is running; finish the import and refresh first",
                        row.path()
                    )));
                }
            }
        }
        Ok(())
    }

    pub(super) fn require_clean_source_relocation(
        &self,
        expected: &ProjectActivationIdentity,
        source_uuid: AssetUuid,
        target: &AssetUri,
    ) -> Result<(), CoreError> {
        let state = self.read_state_recovering_poison();
        if state
            .project
            .as_ref()
            .is_none_or(|project| project.activation_identity() != expected)
        {
            return Err(mutation_error(
                "asset rename belongs to an inactive Editor project",
            ));
        }
        let index = state
            .asset_index
            .as_ref()
            .ok_or_else(|| mutation_error("asset rename requires the current asset index"))?;
        let index = index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if index.project_activation_identity() != Some(expected) {
            return Err(mutation_error(
                "asset rename index belongs to an inactive project",
            ));
        }
        let preflight = AssetMutationRelocationPreflight::evaluate(
            index.runtime_registry(),
            source_uuid,
            target.clone(),
        );
        for asset in preflight
            .companions()
            .iter()
            .chain(preflight.referencer_closure())
        {
            if let Some(row) = index.row_by_uuid(asset.uuid()) {
                if row.import_state() == EditorAssetImportState::Importing {
                    return Err(mutation_error(format!(
                        "asset rename cannot change {} while an import is running; finish the import and refresh first", row.path()
                    )));
                }
            }
        }
        Ok(())
    }
}

fn folder_contains(folder: &AssetUri, locator: &AssetUri) -> bool {
    folder.scheme() == ResourceScheme::Res
        && locator.scheme() == ResourceScheme::Res
        && (locator.path() == folder.path()
            || locator
                .path()
                .strip_prefix(folder.path())
                .is_some_and(|suffix| suffix.starts_with('/')))
}

pub(super) fn mutation_error(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(EDITOR_ASSET_MANAGER_NAME.to_owned(), message.into())
}
