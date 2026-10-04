use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::{
    layout::{LayoutCommand, MainPageId},
    view::ViewInstanceId,
};

use super::super::{callback_dispatch, close_prompt::FloatingClosePermit, RetainedEditorHost};
use crate::ui::retained_host::event_bridge::{apply_record_effects, UiHostEventEffects};

impl RetainedEditorHost {
    pub(super) fn close_floating_window_without_prompt(
        &mut self,
        window_id: &MainPageId,
        instance_ids: Vec<ViewInstanceId>,
    ) -> CloseRequestResponse {
        let command = LayoutCommand::CloseViews {
            window_id: window_id.clone(),
            instance_ids,
        };
        match callback_dispatch::dispatch_layout_command(&self.runtime, command) {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => {
                self.set_status_line(error);
                return CloseRequestResponse::KeepWindowShown;
            }
        }
        self.recompute_if_dirty();
        if self.runtime.floating_window_exists(window_id) {
            CloseRequestResponse::KeepWindowShown
        } else {
            CloseRequestResponse::HideWindow
        }
    }

    pub(super) fn close_floating_window_with_discard(
        &mut self,
        permit: FloatingClosePermit,
    ) -> CloseRequestResponse {
        let (window_id, instance_ids, discard) = permit.into_parts();
        let result = self
            .runtime
            .dispatch_authorized_close_views(window_id.clone(), instance_ids, discard)
            .map(|record| {
                let mut effects = UiHostEventEffects::default();
                apply_record_effects(&mut effects, &record);
                effects
            })
            .map_err(|error| error.to_string());
        match result {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => {
                self.set_status_line(error);
                return CloseRequestResponse::KeepWindowShown;
            }
        }

        self.recompute_if_dirty();
        let window_still_exists = self.runtime.floating_window_exists(&window_id);
        if window_still_exists {
            CloseRequestResponse::KeepWindowShown
        } else {
            CloseRequestResponse::HideWindow
        }
    }

    pub(super) fn floating_window_close_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Option<Vec<ViewInstanceId>> {
        self.runtime.floating_window_instance_ids(window_id)
    }
}

#[cfg(test)]
#[path = "tests/floating_window_optimization_batch_20260830bp_editor_tests.rs"]
mod optimization_batch_20260830bp_editor_tests;
