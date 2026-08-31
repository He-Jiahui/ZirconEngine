use crate::core::asset::AssetWriteAccess;
use crate::core::commands::CommandEvalCtx;
use crate::core::editor_message::PlayStateKind;
use crate::core::editor_operation::EditorOperationSource;
use crate::core::play::PlayModeKind;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::EditorHostEventController;

/// Projects UI-owned state into the neutral core DTO stored by `EditorContext`.
pub(crate) fn command_eval_ctx_from_chrome<I, S>(
    chrome: &EditorChromeSnapshot,
    play_mode: PlayModeKind,
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
        .with_asset_write_access(
            chrome
                .asset_browser
                .selection
                .source_authority()
                .map(|authority| authority.write_access())
                .unwrap_or(AssetWriteAccess::ReadOnly),
        )
        .with_play_state(play_state_from_mode(play_mode))
        .with_capabilities(capabilities)
}

const fn play_state_from_mode(play_mode: PlayModeKind) -> PlayStateKind {
    match play_mode {
        PlayModeKind::Edit => PlayStateKind::Edit,
        PlayModeKind::Building => PlayStateKind::Building,
        PlayModeKind::Playing => PlayStateKind::Playing,
        PlayModeKind::CleanupFailed => PlayStateKind::CleanupFailed,
    }
}

impl EditorHostEventController {
    pub(crate) fn project_command_eval_snapshot(
        &self,
        chrome: &EditorChromeSnapshot,
    ) -> CommandEvalCtx {
        let shell = self.shell().lock();
        let capabilities = shell
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        let context = shell
            .state
            .project_command_eval_ctx(command_eval_ctx_from_chrome(
                chrome,
                self.play_sessions().mode(),
                capabilities,
            ));
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
                    .with_play_state(play_state_from_mode(self.play_sessions().mode()))
            }
            EditorOperationSource::Menu | EditorOperationSource::UiBinding => {
                self.context().command_eval().snapshot()
            }
        }
    }
}
