use super::*;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    surface::UiSurfaceFrame,
    window::{UiWindowEventMetadata, UiWindowInputPumpEvent},
};

type Feedback = std::collections::BTreeMap<String, std::collections::BTreeMap<String, toml::Value>>;
type GeneratedEditorEffects = (UiHostEventEffects, Option<String>);

fn dispatch_generated_binding(
    controller: &crate::ui::host::EditorHostEventController,
    binding: &EditorUiBinding,
) -> Result<(GeneratedEditorEffects, Feedback), String> {
    let EditorUiBindingPayload::MenuAction { action_id } = binding.payload() else {
        return Err("Generated authored association is not a canonical MenuAction".to_string());
    };
    if !action_id.starts_with("workbench.generated_bottom.")
        || !crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action(
            action_id,
        )
    {
        return Err(
            "Generated action is outside the existing preview dispatcher admission".to_string(),
        );
    }
    // This early preview path only observes/records. No pane runtime or Host is captured here.
    let effects = callback_dispatch::dispatch_template_binding_with_arguments(
        controller,
        binding.clone(),
        Vec::new(),
    )?;
    let (updates, status) = callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::generated_bottom_pane_feedback_attributes(action_id);
    Ok(((effects, status), updates))
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_generated_bottom_pane_input(
        &mut self,
        pane_id: &str,
        source: &Arc<UiSurfaceFrame>,
        event: UiWindowInputPumpEvent,
    ) -> bool {
        if !self
            .builtin_template_runtime
            .owns_generated_bottom_pane(pane_id)
        {
            return false;
        }
        let activity = matches!(&event, UiWindowInputPumpEvent::Input(_));
        let result = {
            let controller = &self.runtime;
            self.builtin_template_runtime
                .dispatch_generated_bottom_pane_input(pane_id, source, event, |binding| {
                    dispatch_generated_binding(controller, binding)
                })
        };
        // The Registry/Surface lease is released before effects, status, or projection can reenter it.
        match result {
            Ok(input) => {
                for (effects, status) in input.dispatched {
                    self.apply_dispatch_effects(effects);
                    if let Some(status) = status {
                        self.set_status_line(status);
                    }
                }
                if activity {
                    self.focus_callback_source_window();
                }
                if !self.mark_presentation_dirty_for_pane(pane_id) {
                    self.mark_presentation_dirty();
                }
                self.recompute_if_dirty();
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
        true
    }

    pub(in crate::ui::retained_host::app) fn dispatch_generated_bottom_action_token(
        &mut self,
        control_id: &str,
        token: &str,
        edited_value: Option<&str>,
    ) -> bool {
        if !token.starts_with("template-v2/") || !token.contains("/generated-bottom/e") {
            return false;
        }
        let result = {
            let controller = &self.runtime;
            self.builtin_template_runtime
                .dispatch_generated_bottom_action_token(
                    token,
                    control_id,
                    edited_value,
                    |binding| dispatch_generated_binding(controller, binding),
                )
        };
        if let Some(result) = result {
            match result {
                Ok((effects, status)) => {
                    self.apply_dispatch_effects(effects);
                    if let Some(status) = status {
                        self.set_status_line(status);
                    }
                    self.mark_presentation_dirty();
                    self.recompute_if_dirty();
                }
                Err(error) => self.set_status_line(error.to_string()),
            }
        }
        // Recognized but stale tokens cannot fall through to another owner or raw MenuAction.
        true
    }

    pub(in crate::ui::retained_host::app) fn reconcile_generated_bottom_panes(&mut self) -> bool {
        let panes = self.builtin_template_runtime.generated_bottom_pane_ids();
        if panes.is_empty() {
            return false;
        }
        let mut republish = false;
        for pane_id in panes {
            let is_open = self
                .editor_manager
                .enter()
                .map(|manager| manager.view_host_for_instance_key(&pane_id).is_some());
            match is_open {
                Ok(false) => self
                    .builtin_template_runtime
                    .remove_template_actions_for_pane(&pane_id),
                Ok(true) => {
                    let visible = self.ui.retained_pane_input_source_visible(&pane_id)
                        || self
                            .committed_shell_state
                            .as_ref()
                            .is_some_and(|committed| {
                                committed.model.floating_windows.iter().any(|floating| {
                                    self.native_window_presenters
                                        .window(&floating.window_id)
                                        .is_some_and(|window| {
                                            window.retained_pane_input_source_visible(&pane_id)
                                        })
                                })
                            });
                    if !visible {
                        if let Err(error) = self
                            .builtin_template_runtime
                            .deactivate_generated_bottom_pane_input(
                                &pane_id,
                                UiWindowEventMetadata::default(),
                            )
                        {
                            self.set_status_line(error.to_string());
                        }
                    } else if !self
                        .builtin_template_runtime
                        .generated_bottom_pane_visible(&pane_id)
                    {
                        self.mark_presentation_dirty();
                        republish = true;
                    }
                }
                Err(error) => self.set_status_line(error.to_string()),
            }
        }
        republish
    }
}
