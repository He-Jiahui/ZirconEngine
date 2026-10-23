use std::time::{Duration, Instant};

use crate::ui::retained_host::event_bridge::apply_record_effects;

use super::{RetainedEditorHost, UiHostEventEffects};

impl RetainedEditorHost {
    /// Pending backend work never enters the normal tick's gateway/transition locks.
    pub(super) fn poll_async_play_command(&mut self) -> bool {
        if let Some(result) = self.runtime.take_completed_play_command() {
            match result {
                Ok(record) => {
                    let mut effects = UiHostEventEffects::default();
                    apply_record_effects(&mut effects, &record);
                    self.apply_dispatch_effects(effects);
                }
                Err(error) => {
                    self.set_status_line(error);
                    // The worker already wrote this error into the shared shell. Even
                    // an unchanged status value must now reach the native presentation.
                    self.mark_presentation_dirty();
                }
            }
            self.publish_play_controls();
            self.ui
                .update_host_presentation(|presentation| presentation.play_transition = None);
            self.request_scene_interaction_redraw();
        }
        let Some(label) = self.runtime.pending_play_label() else {
            if self.close_after_play_transition {
                self.close_after_play_transition = false;
                self.native_main_window_close_requested();
            }
            return false;
        };
        self.publish_play_controls();
        let progress = self.runtime.context().i18n().translate(label).replace(
            "{seconds}",
            &self.runtime.pending_play_elapsed().as_secs().to_string(),
        );
        let generation = self.ui.get_host_presentation_generation();
        let changed = generation.structure().play_transition.as_deref() != Some(progress.as_str());
        drop(generation);
        if changed {
            self.ui.update_host_presentation(|presentation| {
                presentation.play_transition = Some(progress)
            });
            self.request_scene_interaction_redraw();
        }
        self.ui
            .set_lifecycle_frame_update(Some(Instant::now() + Duration::from_millis(33)));
        self.publish_interaction_evidence();
        true
    }

    fn publish_play_controls(&mut self) {
        // Read only the live controller mode and the stored command context; a
        // pending backend operation must not require a shell/chrome rebuild.
        if let Ok(context) = self.runtime.command_eval_ctx_for_source(
            &crate::core::editor_operation::EditorOperationSource::UiBinding,
        ) {
            let controls = crate::ui::workbench::model::RunControlsModel::from_commands(
                &self.runtime.commands().lock(),
                &context,
            );
            if let Err(error) = self
                .workbench_window_bridge
                .apply_run_controls(controls)
                .and_then(|_| self.workbench_window_bridge.refresh_prepared_state_change())
            {
                self.set_status_line(error.to_string());
            }
            match self.apply_workbench_projection_presentation() {
                Ok(true) => {}
                Ok(false) => {
                    // Visibility/label changes can require a full native node
                    // projection. Publish it directly while the backend worker
                    // owns the transition; normal shell recompute must wait.
                    self.ui.update_host_presentation(|presentation| {
                        presentation.workbench_window_nodes = crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
                            Some(self.workbench_window_bridge.host_projection()),
                            Some(&presentation.workbench_window_nodes),
                            self.workbench_window_bridge.layout_frames().mount_frame,
                            self.workbench_window_bridge.presentation_scale_factor(),
                        );
                    });
                    self.invalidate_committed_shell_presentation();
                    self.workbench_window_bridge
                        .mark_host_projection_committed();
                }
                Err(error) => self.set_status_line(error.to_string()),
            }
            if std::env::var_os("ZIRCON_TRACE_EDITOR_INTERACTION").is_some() {
                static MODE: std::sync::Mutex<Option<crate::core::editor_message::PlayStateKind>> =
                    std::sync::Mutex::new(None);
                let mut mode = MODE.lock().unwrap_or_else(|e| e.into_inner());
                if *mode != Some(controls.state) {
                    *mode = Some(controls.state);
                    for node in self
                        .ui
                        .get_host_presentation()
                        .workbench_window_nodes
                        .iter()
                        .filter(|n| {
                            n.control_id.as_str() == "WorkbenchRunStop"
                                || n.control_id.as_str() == "WorkbenchRunPlay"
                        })
                    {
                        eprintln!("editor_interaction kind=native_run_control mode={:?} control={} icon={:?} text={:?} disabled={} frame={:?}", controls.state, node.control_id, node.icon_name, node.text, node.disabled, (node.frame.x, node.frame.y, node.frame.width, node.frame.height));
                    }
                }
            }
        }
    }
}
