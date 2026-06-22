use crate::ui::retained_host::primitives::SharedString;

use super::super::super::data::FrameRect;
use super::super::UiHostWindow;

impl UiHostWindow {
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
}
