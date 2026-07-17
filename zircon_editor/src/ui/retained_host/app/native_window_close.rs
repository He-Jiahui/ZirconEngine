use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::layout::MainPageId;

use super::close_prompt::ClosePromptTarget;
use super::*;

mod floating_window;
mod prompt_actions;

impl RetainedEditorHost {
    pub(super) fn native_main_window_close_requested(&mut self) -> CloseRequestResponse {
        self.recompute_if_dirty();
        let instances = self.runtime.current_view_instances();
        let dirty = close_prompt::all_dirty_close_views(&instances);
        if !dirty.is_empty() {
            let close_instances = instances
                .into_iter()
                .map(|instance| instance.instance_id)
                .collect();
            self.begin_close_prompt(ClosePromptTarget::MainWindow, close_instances, dirty);
            return CloseRequestResponse::KeepWindowShown;
        }
        CloseRequestResponse::HideWindow
    }

    pub(super) fn native_floating_window_close_requested(
        &mut self,
        window_id: &MainPageId,
    ) -> CloseRequestResponse {
        self.recompute_if_dirty();
        let Some(instance_ids) = self.floating_window_close_instance_ids(window_id) else {
            return CloseRequestResponse::KeepWindowShown;
        };

        let dirty = close_prompt::dirty_close_views(
            &self.runtime.current_view_instances(),
            instance_ids.clone(),
        );
        if !dirty.is_empty() {
            self.begin_close_prompt(
                ClosePromptTarget::FloatingWindow(window_id.clone()),
                instance_ids,
                dirty,
            );
            return CloseRequestResponse::KeepWindowShown;
        }

        self.close_floating_window_without_prompt(window_id, instance_ids)
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn main_window_close_reuses_the_view_instance_snapshot() {
        let source = include_str!("native_window_close.rs");
        let production = source.split("#[cfg(test)]").next().expect("implementation");
        let snapshot_call = ["current_view_", "instances()"].concat();

        assert_eq!(production.matches(&snapshot_call).count(), 2);
        assert!(production.contains("all_dirty_close_views(&instances)"));
    }
}
