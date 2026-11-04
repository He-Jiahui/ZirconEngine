use zircon_runtime::asset::pipeline::manager::ProjectGenerationMatch;
use zircon_runtime::asset::project::ProjectPaths;

use crate::core::asset::EditorAssetFolderOperation;
use crate::ui::binding::EditorUiEventKind;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::HostInvalidationMask;

use super::super::RetainedEditorHost;
use super::session::{
    AssetNamePromptRequest, AssetNamePromptSession, PROMPT_COMMIT_ID, PROMPT_SOURCE,
};

const PROMPT_CONTROL: &str = "WorkbenchCommandPalette";
const QUERY_BINDING: &str = "CommandPalette/QueryChanged";
const WINDOW_BINDING: &str = "CommandPalette/WindowRequested";
const COMMIT_BINDING: &str = "CommandPalette/Commit";

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn request_folder_create(
        &mut self,
        parent_folder_id: &str,
    ) -> Result<(), String> {
        let request = AssetNamePromptRequest::create_folder(parent_folder_id)
            .map_err(|key| self.asset_name_prompt_message(key))?;
        self.open_asset_name_prompt(request)
    }

    pub(in crate::ui::retained_host::app) fn request_folder_rename(
        &mut self,
        folder_id: &str,
    ) -> Result<(), String> {
        let request = AssetNamePromptRequest::rename_folder(folder_id)
            .map_err(|key| self.asset_name_prompt_message(key))?;
        self.open_asset_name_prompt(request)
    }

    pub(in crate::ui::retained_host::app) fn request_asset_rename(
        &mut self,
        asset_uuid: &str,
    ) -> Result<(), String> {
        let catalog = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .catalog_snapshot();
        let asset = catalog
            .asset(asset_uuid)
            .ok_or_else(|| self.asset_name_prompt_message("asset.name_prompt.target_changed"))?;
        let request = AssetNamePromptRequest::rename_asset(asset_uuid, &asset.locator)
            .map_err(|key| self.asset_name_prompt_message(key))?;
        self.open_asset_name_prompt(request)
    }

    fn open_asset_name_prompt(&mut self, request: AssetNamePromptRequest) -> Result<(), String> {
        let catalog = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .catalog_snapshot();
        let manager = self
            .asset_runtime_access
            .project_asset_manager()
            .map_err(|error| error.to_string())?;
        let snapshot = manager
            .current_project_generation_snapshot()
            .ok_or_else(|| self.asset_name_prompt_message("asset.name_prompt.target_changed"))?;
        let (_, generation) = snapshot.into_parts();
        drop(manager);
        if ProjectPaths::display_path(generation.project_root())
            .to_string_lossy()
            .as_ref()
            != catalog.project_root.as_ref()
            || !request.source_is_current(&catalog)
        {
            return Err(self.asset_name_prompt_message("asset.name_prompt.target_changed"));
        }
        let session = AssetNamePromptSession {
            generation,
            catalog,
            request,
        };
        let i18n = self.runtime.context().i18n();
        let state = session.state(session.request.initial_query(), i18n);
        let title = i18n.translate(session.request.title_key());
        let placeholder = i18n.translate(session.request.placeholder_key());
        let empty = i18n.translate("asset.name_prompt.empty");
        let help = i18n
            .translate("asset.name_prompt.help")
            .replace("{source}", &session.request.source().to_string());
        let opened = self
            .workbench_window_bridge
            .open_command_palette_with_chrome(
                state,
                PROMPT_SOURCE,
                &placeholder,
                &empty,
                &title,
                &help,
            )
            .map_err(|error| error.to_string())?;
        if !opened {
            return Err(self.asset_name_prompt_message("asset.name_prompt.unavailable"));
        }
        self.scene_picker_session = None;
        self.asset_name_prompt = Some(session);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn dispatch_workbench_asset_name_prompt_edited(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if control_id != PROMPT_CONTROL
            || !self.workbench_window_bridge.command_palette_open()
            || self.workbench_window_bridge.command_palette_source() != PROMPT_SOURCE
        {
            return None;
        }
        let expected_kind = match binding_id {
            QUERY_BINDING | WINDOW_BINDING => EditorUiEventKind::Change,
            COMMIT_BINDING => EditorUiEventKind::Submit,
            _ => return None,
        };
        if self
            .workbench_window_bridge
            .binding_by_id(binding_id)
            .is_none_or(|binding| binding.path().event_kind != expected_kind)
        {
            return None;
        }
        if binding_id == COMMIT_BINDING {
            return Some(self.submit_asset_name_prompt(value));
        }
        if binding_id == WINDOW_BINDING {
            // A name prompt has at most one candidate and no additional result window.
            return Some(Ok(UiHostEventEffects::default()));
        }
        let Some(session) = self.asset_name_prompt.as_ref() else {
            return Some(Err(
                self.asset_name_prompt_message("asset.name_prompt.target_changed")
            ));
        };
        let state = session.state(value, self.runtime.context().i18n());
        Some(
            self.workbench_window_bridge
                .update_command_palette_query(state)
                .map_err(|error| error.to_string())
                .map(|changed| {
                    let mut effects = UiHostEventEffects::default();
                    if changed {
                        effects.request_paint_only();
                    }
                    effects
                }),
        )
    }

    fn submit_asset_name_prompt(&mut self, command_id: &str) -> Result<UiHostEventEffects, String> {
        if command_id != PROMPT_COMMIT_ID
            || self.workbench_window_bridge.command_palette_window_offset() != Some(0)
        {
            return Err(self.asset_name_prompt_message("asset.name_prompt.invalid_submission"));
        }
        let session = self
            .asset_name_prompt
            .as_ref()
            .ok_or_else(|| self.asset_name_prompt_message("asset.name_prompt.target_changed"))?;
        let manager = self
            .asset_runtime_access
            .project_asset_manager()
            .map_err(|error| error.to_string())?;
        if manager.check_project_generation(&session.generation) != ProjectGenerationMatch::Current
        {
            return Err(self.asset_name_prompt_message("asset.name_prompt.target_changed"));
        }
        drop(manager);
        let catalog = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .catalog_snapshot();
        // Folder-only operations may leave the asset input sequence unchanged. Revalidate the
        // published directory topology as well, while allowing thumbnail-only publications.
        if catalog.project_root != session.catalog.project_root
            || !session.request.source_is_current(&catalog)
            || catalog.folders.as_ref() != session.catalog.folders.as_ref()
        {
            return Err(self.asset_name_prompt_message("asset.name_prompt.target_changed"));
        }
        let target = session
            .candidate(&self.workbench_window_bridge.command_palette_query())
            .map_err(|key| self.asset_name_prompt_message(key))?
            .to_string();
        let request = session.request.clone();
        match request {
            AssetNamePromptRequest::CreateFolder { .. } => {
                self.request_folder_mutation(EditorAssetFolderOperation::Create, &target, None)?;
            }
            AssetNamePromptRequest::RenameFolder { source } => {
                self.request_folder_mutation(
                    EditorAssetFolderOperation::Rename,
                    &source.to_string(),
                    Some(&target),
                )?;
            }
            AssetNamePromptRequest::RenameAsset { asset_uuid, .. } => {
                self.request_asset_relocation(&asset_uuid, &target)?;
            }
        }
        self.cancel_asset_name_prompt();
        self.workbench_window_bridge
            .close_command_palette()
            .map_err(|error| error.to_string())?;
        let mut effects = UiHostEventEffects::default();
        effects.request_render_and_presentation();
        Ok(effects)
    }

    pub(in crate::ui::retained_host::app) fn cancel_asset_name_prompt(&mut self) {
        self.asset_name_prompt = None;
    }

    fn asset_name_prompt_message(&self, key: &str) -> String {
        self.runtime.context().i18n().translate(key).to_string()
    }
}
