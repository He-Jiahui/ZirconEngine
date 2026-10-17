use super::*;
use crate::ui::template_runtime::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(super) fn dispatch_workbench_context_menu_requested(
        &mut self,
        mut request: WorkbenchContextMenuRequestData,
    ) {
        self.focus_callback_source_window();
        if !(match self
            .active_activity_window_template_document_is(UNREAL_EDITOR_WINDOW_DOCUMENT_ID)
        {
            Ok(value) => value,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        }) {
            return;
        }

        project_keep_play_changes_context_item(
            &mut request,
            self.runtime.play_sessions().mode() == crate::core::play::PlayModeKind::Playing,
        );

        match self.workbench_window_bridge.open_context_menu(&request) {
            Ok(true) => {
                self.apply_dispatch_effects(UiHostEventEffects::default());
                self.set_status_line(format!(
                    "Context menu opened for {}",
                    request.target_value_text
                ));
            }
            Ok(false) => self.set_status_line("Workbench context menu is not available"),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

fn project_keep_play_changes_context_item(
    request: &mut WorkbenchContextMenuRequestData,
    playing: bool,
) {
    if !playing
        || !request
            .target_path
            .as_str()
            .starts_with("workbench://scene/")
    {
        return;
    }
    let item = "Keep Play Changes|action=menu.item.keep_play_changes,icon=save".into();
    let index = request
        .menu_items
        .iter()
        .position(|item| item.as_str() == "---")
        .unwrap_or(request.menu_items.len());
    request.menu_items.insert(index, item);
}

#[cfg(test)]
#[path = "tests/context_menu.rs"]
mod tests;
