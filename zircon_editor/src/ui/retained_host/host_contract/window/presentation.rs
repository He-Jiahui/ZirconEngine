use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::super::data::{
    FrameRect, HostClosePromptData, HostMenuStateData, HostPaneInteractionStateData,
    HostWindowBootstrapData, HostWindowPresentationData,
};
use super::super::globals::HostContractState;
use super::super::redraw::HostRedrawRequest;
use super::{template_hover, UiHostWindow};

impl UiHostWindow {
    pub(crate) fn set_host_presentation(&self, presentation: HostWindowPresentationData) {
        let mut state = self.state.borrow_mut();
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        if state.presentation_rebuild_count <= 8
            || state.presentation_rebuild_count.is_power_of_two()
        {
            if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
                write_diagnostic_log(
                    "editor_host_window",
                    format!(
                        "set_host_presentation count={} project_path={} viewport_label={} status={} center={} document={} viewport={}",
                        state.presentation_rebuild_count,
                        presentation.host_shell.project_path,
                        presentation.host_shell.viewport_label,
                        presentation.host_shell.status_secondary,
                        frame_summary(&presentation.host_layout.center_band_frame),
                        frame_summary(&presentation.host_layout.document_region_frame),
                        frame_summary(&presentation.host_layout.viewport_content_frame)
                    ),
                );
            }
        }
        state.host_presentation = presentation;
    }

    pub(crate) fn get_host_presentation(&self) -> HostWindowPresentationData {
        let state = self.state.borrow();
        host_presentation_from_state(&state)
    }

    pub(crate) fn set_close_prompt(&self, prompt: HostClosePromptData) {
        let damage = {
            let mut state = self.state.borrow_mut();
            let current = state.host_presentation.close_prompt.clone();
            let damage = if current.visible {
                current.overlay_frame
            } else {
                prompt.overlay_frame.clone()
            };
            state.host_presentation.close_prompt = prompt;
            damage
        };
        self.queue_external_redraw(HostRedrawRequest::region(damage));
    }

    pub(crate) fn clear_close_prompt(&self) {
        self.set_close_prompt(HostClosePromptData::default());
    }

    pub(crate) fn get_menu_state(&self) -> HostMenuStateData {
        self.state.borrow().menu_state.clone()
    }

    pub(crate) fn get_pane_interaction_state(&self) -> HostPaneInteractionStateData {
        self.state.borrow().pane_interaction_state.clone()
    }

    pub(crate) fn set_hovered_template_node_for_pointer_move(
        &self,
        control_id: SharedString,
        frame: FrameRect,
    ) {
        let mut state = self.state.borrow_mut();
        state.pane_interaction_state.hovered_template_control_id = control_id;
        state
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .clear();
        state
            .pane_interaction_state
            .hovered_template_action_id
            .clear();
        state
            .pane_interaction_state
            .hovered_template_value_text
            .clear();
        state.pane_interaction_state.hovered_template_frame = frame;
    }

    pub(crate) fn set_hovered_template_row_for_pointer_move(
        &self,
        control_id: SharedString,
        dispatch_kind: SharedString,
        action_id: SharedString,
        value_text: SharedString,
        frame: FrameRect,
    ) {
        let mut state = self.state.borrow_mut();
        state.pane_interaction_state.hovered_template_control_id = control_id;
        state.pane_interaction_state.hovered_template_dispatch_kind = dispatch_kind;
        state.pane_interaction_state.hovered_template_action_id = action_id;
        state.pane_interaction_state.hovered_template_value_text = value_text;
        state.pane_interaction_state.hovered_template_frame = frame;
    }

    pub(crate) fn clear_hovered_template_node_for_pointer_move(&self) {
        let mut state = self.state.borrow_mut();
        state
            .pane_interaction_state
            .hovered_template_control_id
            .clear();
        state
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .clear();
        state
            .pane_interaction_state
            .hovered_template_action_id
            .clear();
        state
            .pane_interaction_state
            .hovered_template_value_text
            .clear();
        state.pane_interaction_state.hovered_template_frame = FrameRect::default();
    }

    pub(crate) fn get_host_window_bootstrap(&self) -> HostWindowBootstrapData {
        let state = self.state.borrow();
        HostWindowBootstrapData {
            shell_frame: FrameRect {
                x: 0.0,
                y: 0.0,
                width: state.window_size.width as f32,
                height: state.window_size.height as f32,
            },
            viewport_content_frame: state
                .host_presentation
                .host_layout
                .viewport_content_frame
                .clone(),
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn host_presentation_from_state(
    state: &HostContractState,
) -> HostWindowPresentationData {
    let mut presentation = state.host_presentation.clone();
    presentation.menu_state = state.menu_state.clone();
    presentation.pane_interaction_state = state.pane_interaction_state.clone();
    presentation.text_input_focus = state.text_input_focus.clone();
    presentation.viewport_image = state.viewport_image.clone();
    template_hover::apply_template_hover_to_presentation(
        &mut presentation,
        &state.pane_interaction_state,
    );
    presentation
}

fn frame_summary(frame: &FrameRect) -> String {
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        frame.x, frame.y, frame.width, frame.height
    )
}
