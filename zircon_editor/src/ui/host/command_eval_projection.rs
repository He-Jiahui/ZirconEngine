use crate::core::asset::AssetWriteAccess;
use crate::core::commands::CommandEvalCtx;
use crate::core::editor_message::PlayStateKind;
use crate::core::editor_operation::EditorOperationSource;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::startup::EditorSessionMode;

use super::EditorHostEventController;

/// Projects UI-owned state into the neutral core DTO stored by `EditorContext`.
pub(crate) fn command_eval_ctx_from_chrome<I, S>(
    chrome: &EditorChromeSnapshot,
    capabilities: I,
) -> CommandEvalCtx
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    CommandEvalCtx::interactive()
        .with_optional_focused_document_kind(chrome.focused_document_kind.clone())
        .with_project_open(chrome.project_open)
        .with_undo_available(chrome.can_undo)
        .with_redo_available(chrome.can_redo)
        .with_selection_count(if chrome.inspector.is_some() { 1 } else { 0 })
        .with_asset_write_access(
            chrome
                .asset_browser
                .selection
                .source_authority()
                .map(|authority| authority.write_access())
                .unwrap_or(AssetWriteAccess::ReadOnly),
        )
        .with_play_state(match chrome.session_mode {
            EditorSessionMode::Playing => PlayStateKind::Playing,
            EditorSessionMode::Welcome | EditorSessionMode::Project => PlayStateKind::Edit,
        })
        .with_capabilities(capabilities)
}

impl EditorHostEventController {
    pub(crate) fn project_command_eval_snapshot(
        &self,
        chrome: &EditorChromeSnapshot,
    ) -> CommandEvalCtx {
        let capabilities = self
            .shell()
            .lock()
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        let context = command_eval_ctx_from_chrome(chrome, capabilities);
        self.context().command_eval().replace(context.clone());
        context
    }

    pub(crate) fn command_eval_ctx_for_source(
        &self,
        source: &EditorOperationSource,
    ) -> CommandEvalCtx {
        match source {
            EditorOperationSource::Remote | EditorOperationSource::Cli => {
                let capabilities = self
                    .shell()
                    .lock()
                    .manager
                    .capability_snapshot()
                    .enabled_capabilities()
                    .to_vec();
                CommandEvalCtx::headless(capabilities)
            }
            EditorOperationSource::Menu | EditorOperationSource::UiBinding => {
                self.context().command_eval().snapshot()
            }
        }
    }
}
