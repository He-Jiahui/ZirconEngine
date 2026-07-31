use crate::ui::retained_host::callback_dispatch::dispatch_builtin_floating_window_focus_for_source;
use crate::ui::workbench::layout::MainPageId;

use super::super::super::super::{RetainedEditorHost, workbench_snapshot_access};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn with_callback_source_window<T>(
        &mut self,
        source_window_id: Option<MainPageId>,
        callback: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.callback_source_window.clone();
        self.callback_source_window = source_window_id;
        let result = callback(self);
        self.callback_source_window = previous;
        result
    }

    pub(in crate::ui::retained_host::app) fn focus_callback_source_window(&mut self) {
        let source_window_id = self.callback_source_window.clone();
        let Some(source_window_id) = source_window_id else {
            self.last_focused_callback_window = None;
            return;
        };

        match dispatch_builtin_floating_window_focus_for_source(
            &self.runtime,
            Some(&source_window_id),
            self.last_focused_callback_window.as_ref(),
        ) {
            Some(Ok(effects)) => {
                self.apply_dispatch_effects(effects);
                self.last_focused_callback_window = Some(source_window_id);
            }
            Some(Err(error)) => self.set_status_line(error),
            None => {
                self.last_focused_callback_window = Some(source_window_id);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn note_focused_floating_window(
        &mut self,
        window_id: Option<MainPageId>,
    ) {
        self.last_focused_callback_window = window_id;
    }

    pub(in crate::ui::retained_host::app) fn note_focused_floating_window_surface(
        &mut self,
        surface_key: &str,
    ) {
        if surface_key == "main" {
            self.last_focused_callback_window = None;
            return;
        }

        let chrome = self.runtime.chrome_snapshot();
        self.last_focused_callback_window =
            workbench_snapshot_access::floating_window_id_for_surface_key(
                &chrome.workbench,
                surface_key,
            );
    }
}
