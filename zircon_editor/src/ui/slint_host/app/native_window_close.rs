use slint::CloseRequestResponse;

use crate::ui::workbench::layout::{DocumentNode, LayoutCommand, MainPageId};
use crate::ui::workbench::view::ViewInstanceId;

use super::close_prompt::{ClosePromptTarget, PendingClosePrompt};
use super::*;

impl SlintEditorHost {
    pub(super) fn native_main_window_close_requested(&mut self) -> CloseRequestResponse {
        self.recompute_if_dirty();
        let dirty = close_prompt::all_dirty_close_views(&self.runtime.current_view_instances());
        if !dirty.is_empty() {
            let close_instances = self
                .runtime
                .current_view_instances()
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

    pub(super) fn close_prompt_action_clicked(&mut self, action_id: &str) {
        let Some(action) = close_prompt::close_action_id(action_id) else {
            return;
        };
        let Some(prompt) = self.pending_close_prompt.clone() else {
            return;
        };
        match action {
            "cancel" => {
                self.clear_close_prompt(&prompt.target);
                self.pending_close_prompt = None;
            }
            "discard" => {
                self.clear_close_prompt(&prompt.target);
                self.finish_prompted_close(prompt, false);
            }
            "save" => {
                if let Err(error) = self.save_dirty_prompt_views(&prompt.dirty_views) {
                    self.set_status_line(error);
                    self.show_close_prompt(&prompt);
                    return;
                }
                self.clear_close_prompt(&prompt.target);
                self.finish_prompted_close(prompt, true);
            }
            _ => {}
        }
    }

    fn begin_close_prompt(
        &mut self,
        target: ClosePromptTarget,
        close_instances: Vec<ViewInstanceId>,
        dirty_views: Vec<close_prompt::DirtyCloseView>,
    ) {
        let prompt = PendingClosePrompt::new(target, close_instances, dirty_views);
        self.show_close_prompt(&prompt);
        self.pending_close_prompt = Some(prompt);
    }

    fn show_close_prompt(&self, prompt: &PendingClosePrompt) {
        let ui = self.close_prompt_ui(&prompt.target);
        close_prompt::show_prompt(&ui, prompt);
    }

    fn clear_close_prompt(&self, target: &ClosePromptTarget) {
        let ui = self.close_prompt_ui(target);
        close_prompt::clear_prompt(&ui);
    }

    fn close_prompt_ui(&self, target: &ClosePromptTarget) -> UiHostWindow {
        match target {
            ClosePromptTarget::MainWindow => self.ui.clone_strong(),
            ClosePromptTarget::FloatingWindow(window_id) => self
                .native_window_presenters
                .window(window_id)
                .unwrap_or_else(|| self.ui.clone_strong()),
        }
    }

    fn finish_prompted_close(&mut self, prompt: PendingClosePrompt, _saved: bool) {
        self.pending_close_prompt = None;
        match prompt.target {
            ClosePromptTarget::MainWindow => self.ui.request_exit(),
            ClosePromptTarget::FloatingWindow(window_id) => {
                let _ =
                    self.close_floating_window_without_prompt(&window_id, prompt.close_instances);
            }
        }
    }

    fn close_floating_window_without_prompt(
        &mut self,
        window_id: &MainPageId,
        instance_ids: Vec<ViewInstanceId>,
    ) -> CloseRequestResponse {
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

    fn save_dirty_prompt_views(
        &self,
        views: &[close_prompt::DirtyCloseView],
    ) -> Result<(), String> {
        for view in views {
            match view.descriptor_id.0.as_str() {
                "editor.ui_asset" => {
                    self.editor_manager
                        .save_ui_asset_editor(&view.instance_id)
                        .map_err(|error| error.to_string())?;
                }
                "editor.animation_sequence" | "editor.animation_graph" => {
                    self.editor_manager
                        .save_animation_editor(&view.instance_id)
                        .map_err(|error| error.to_string())?;
                }
                _ => {
                    return Err(format!(
                        "Cannot save {} automatically; use Discard or Cancel",
                        view.title
                    ));
                }
            }
        }
        Ok(())
    }

    fn floating_window_close_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Option<Vec<ViewInstanceId>> {
        let layout = self.runtime.current_layout();
        let window = layout
            .floating_windows
            .iter()
            .find(|window| &window.window_id == window_id)?;
        let mut instances = Vec::new();
        collect_document_node_instances(&window.workspace, &mut instances);
        (!instances.is_empty()).then_some(instances)
    }
}

fn collect_document_node_instances(node: &DocumentNode, out: &mut Vec<ViewInstanceId>) {
    match node {
        DocumentNode::Tabs(stack) => out.extend(stack.tabs.iter().cloned()),
        DocumentNode::SplitNode { first, second, .. } => {
            collect_document_node_instances(first, out);
            collect_document_node_instances(second, out);
        }
    }
}
