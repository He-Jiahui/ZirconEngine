use slint::CloseRequestResponse;

use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::layout::MainPageId;

use super::*;

impl SlintEditorHost {
    pub(super) fn native_floating_window_close_requested(
        &mut self,
        window_id: &MainPageId,
    ) -> CloseRequestResponse {
        self.recompute_if_dirty();
        let Some(instance_ids) = callback_dispatch::resolve_builtin_floating_window_close_instances(
            &self.runtime,
            window_id,
        ) else {
            return CloseRequestResponse::KeepWindowShown;
        };

        for instance_id in instance_ids {
            match callback_dispatch::dispatch_layout_command(
                &self.runtime,
                LayoutCommand::CloseView { instance_id },
            ) {
                Ok(effects) => self.apply_dispatch_effects(effects),
                Err(error) => {
                    self.set_status_line(error);
                    return CloseRequestResponse::KeepWindowShown;
                }
            }
        }

        self.recompute_if_dirty();
        let window_still_exists = self
            .runtime
            .current_layout()
            .floating_windows
            .iter()
            .any(|window| &window.window_id == window_id);
        if window_still_exists {
            CloseRequestResponse::KeepWindowShown
        } else {
            CloseRequestResponse::HideWindow
        }
    }
}
