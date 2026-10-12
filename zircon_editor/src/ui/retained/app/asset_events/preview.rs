#[path = "preview/nested.rs"]
mod nested;
#[path = "preview/suggestions.rs"]
mod suggestions;
#[path = "preview/value.rs"]
mod value;

use super::*;
use crate::ui::host::EditorError;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn retained_pane_input_windows(
        &self,
    ) -> Vec<crate::ui::retained_host::host_contract::UiHostWindow> {
        let mut windows = Vec::new();
        if self.ui.has_retained_pane_input_address() {
            windows.push(self.ui.clone_strong());
        }
        if let Some(committed) = self.committed_shell_state.as_ref() {
            for floating in &committed.model.floating_windows {
                if let Some(window) = self.native_window_presenters.window(&floating.window_id) {
                    if window.has_retained_pane_input_address() {
                        windows.push(window);
                    }
                }
            }
        }
        windows
    }

    pub(in crate::ui::retained_host::app) fn retire_hidden_retained_pane_inputs(
        &mut self,
        windows: &[crate::ui::retained_host::host_contract::UiHostWindow],
    ) {
        for window in windows {
            let Some((pane_id, metadata)) = window.hidden_retained_pane_input() else {
                continue;
            };
            if self
                .builtin_template_runtime
                .owns_generated_bottom_pane(pane_id.as_str())
            {
                match self
                    .builtin_template_runtime
                    .deactivate_generated_bottom_pane_input(pane_id.as_str(), metadata)
                {
                    Ok(_) => window.clear_retained_pane_input_address(pane_id.as_str()),
                    Err(error) => self.set_status_line(error.to_string()),
                }
                continue;
            }
            let instance_id = ViewInstanceId::new(pane_id.as_str());
            let result = self
                .editor_manager
                .enter()
                .map_err(EditorError::from)
                .and_then(|manager| {
                    manager.deactivate_ui_asset_editor_preview_input(&instance_id, metadata)
                });
            match result {
                Ok(changed) => {
                    // The Runtime owner has released capture/focus/IME before its route is retired.
                    window.clear_retained_pane_input_address(pane_id.as_str());
                    if changed {
                        match self
                            .apply_scoped_ui_asset_presentation(std::slice::from_ref(&instance_id))
                        {
                            Ok(true) => {}
                            Ok(false) => self.mark_presentation_dirty_for_view(&instance_id),
                            Err(error) => {
                                self.mark_presentation_dirty_for_view(&instance_id);
                                self.set_status_line(error.to_string());
                            }
                        }
                    }
                }
                Err(error) => self.set_status_line(error.to_string()),
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn dispatch_ui_asset_preview_input(
        &mut self,
        instance_id: &str,
        source_frame: &std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
        event: zircon_runtime_interface::ui::window::UiWindowInputPumpEvent,
    ) {
        let input_activity = matches!(
            &event,
            zircon_runtime_interface::ui::window::UiWindowInputPumpEvent::Input(_)
        );
        let instance_id = ViewInstanceId::new(instance_id);
        let result = self
            .editor_manager
            .enter()
            .map_err(EditorError::from)
            .and_then(|manager| {
                manager.dispatch_ui_asset_editor_preview_input(&instance_id, source_frame, event)
            });
        match result {
            Ok(true) => {
                if input_activity {
                    self.focus_callback_source_window();
                }
                // Publish the same PreviewHost frame before another normalized input observes it.
                match self.apply_scoped_ui_asset_presentation(std::slice::from_ref(&instance_id)) {
                    Ok(true) => {}
                    Ok(false) => self.mark_presentation_dirty_for_view(&instance_id),
                    Err(error) => {
                        self.mark_presentation_dirty_for_view(&instance_id);
                        self.set_status_line(error.to_string());
                    }
                }
            }
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
