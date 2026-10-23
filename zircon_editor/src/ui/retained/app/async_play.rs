use std::time::{Duration, Instant};

use crate::ui::retained_host::event_bridge::apply_record_effects;

use super::{RetainedEditorHost, UiHostEventEffects};
use crate::core::play::{PlayKind, PlayMode};

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
            self.ui
                .update_host_presentation(|presentation| presentation.play_transition = None);
            // Start/Stop changes the document, toolbar and status together.
            // A preview-only damage rect cannot publish that structural change.
            self.ui.request_full_redraw();
        }
        let Some(label) = self.runtime.pending_play_label() else {
            let generation = self.ui.get_host_presentation_generation();
            let waiting_for_image = match self.runtime.play_sessions().mode_snapshot() {
                PlayMode::Playing {
                    kind: PlayKind::Play,
                } => generation.viewport_images().game().is_none(),
                PlayMode::Playing {
                    kind: PlayKind::Simulate,
                } => generation.viewport_images().simulate().is_none(),
                _ => false,
            };
            let overlay_visible = generation.structure().play_transition.is_some();
            let panel = crate::ui::retained_host::host_contract::transition_panel_frame(
                generation.structure(),
            );
            drop(generation);
            if waiting_for_image {
                self.update_play_transition_overlay(
                    "Preparing Game view…",
                    self.runtime.pending_preview_elapsed(),
                );
                self.ui
                    .set_lifecycle_frame_update(Some(Instant::now() + Duration::from_millis(33)));
            } else if overlay_visible {
                self.ui
                    .update_host_presentation(|presentation| presentation.play_transition = None);
                if let Some(panel) = panel {
                    self.ui.request_redraw_region(panel);
                }
            }
            if self.close_after_play_transition {
                self.close_after_play_transition = false;
                self.native_main_window_close_requested();
            }
            return false;
        };
        // Read only the live controller mode and the stored command context; a
        // pending backend operation must not require a shell/chrome rebuild.
        let needs_controls = self
            .ui
            .get_host_presentation_generation()
            .structure()
            .play_transition
            .as_deref()
            .is_none_or(|text| !text.starts_with(label));
        if needs_controls {
            if let Ok(context) = self.runtime.command_eval_ctx_for_source(
                &crate::core::editor_operation::EditorOperationSource::UiBinding,
            ) {
                let controls = crate::ui::workbench::model::RunControlsModel::from_commands(
                    &self.runtime.commands().lock(),
                    &context,
                );
                if let Err(error) = self.workbench_window_bridge.apply_run_controls(controls) {
                    self.set_status_line(error.to_string());
                }
                if let Err(error) = self.apply_workbench_projection_presentation() {
                    self.set_status_line(error.to_string());
                }
            }
        }
        if needs_controls {
            self.ui.request_full_redraw();
        }
        self.update_play_transition_overlay(label, self.runtime.pending_play_elapsed());
        self.ui
            .set_lifecycle_frame_update(Some(Instant::now() + Duration::from_millis(33)));
        true
    }

    fn update_play_transition_overlay(&mut self, label: &str, elapsed: Duration) {
        let phase = ((elapsed.as_millis() / 33) % 32) as u8;
        let progress = format!("{label}  {}s", elapsed.as_secs());
        let generation = self.ui.get_host_presentation_generation();
        let changed = generation.structure().play_transition.as_deref() != Some(progress.as_str())
            || generation.structure().play_transition_phase != phase;
        drop(generation);
        if changed {
            self.ui.update_host_presentation(|presentation| {
                presentation.play_transition = Some(progress);
                presentation.play_transition_phase = phase;
            });
            let generation = self.ui.get_host_presentation_generation();
            if let Some(panel) = crate::ui::retained_host::host_contract::transition_panel_frame(
                generation.structure(),
            ) {
                self.ui.request_redraw_region(panel);
            }
            self.publish_operator_evidence();
        }
    }
}
