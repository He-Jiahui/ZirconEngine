use super::super::super::data::FrameRect;
use super::super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn set_hovered_template_node_for_pointer_move(
        &self,
        control_id: &str,
        frame: &FrameRect,
    ) {
        {
            let state = self.state.borrow();
            let current = state.pane_interaction_state.as_ref();
            if current.hovered_template_control_id == control_id
                && current.hovered_template_dispatch_kind.is_empty()
                && current.hovered_template_action_id.is_empty()
                && current.hovered_template_value_text.is_empty()
                && current.hovered_template_frame == *frame
            {
                return;
            }
        }
        let mut state = self.state.borrow_mut();
        state.update_pane_interaction(|interaction| {
            interaction.hovered_template_control_id = control_id.to_owned();
            interaction.hovered_template_dispatch_kind.clear();
            interaction.hovered_template_action_id.clear();
            interaction.hovered_template_value_text.clear();
            interaction.hovered_template_frame = frame.clone();
        });
    }

    pub(crate) fn set_hovered_template_row_for_pointer_move(
        &self,
        control_id: &str,
        dispatch_kind: &str,
        action_id: &str,
        value_text: &str,
        frame: &FrameRect,
    ) {
        {
            let state = self.state.borrow();
            let current = state.pane_interaction_state.as_ref();
            if current.hovered_template_control_id == control_id
                && current.hovered_template_dispatch_kind == dispatch_kind
                && current.hovered_template_action_id == action_id
                && current.hovered_template_value_text == value_text
                && current.hovered_template_frame == *frame
            {
                return;
            }
        }
        let mut state = self.state.borrow_mut();
        state.update_pane_interaction(|interaction| {
            interaction.hovered_template_control_id = control_id.to_owned();
            interaction.hovered_template_dispatch_kind = dispatch_kind.to_owned();
            interaction.hovered_template_action_id = action_id.to_owned();
            interaction.hovered_template_value_text = value_text.to_owned();
            interaction.hovered_template_frame = frame.clone();
        });
    }

    pub(crate) fn clear_hovered_template_node_for_pointer_move(&self) {
        {
            let state = self.state.borrow();
            let current = state.pane_interaction_state.as_ref();
            if current.hovered_template_control_id.is_empty()
                && current.hovered_template_dispatch_kind.is_empty()
                && current.hovered_template_action_id.is_empty()
                && current.hovered_template_value_text.is_empty()
                && current.hovered_template_frame == FrameRect::default()
            {
                return;
            }
        }
        let mut state = self.state.borrow_mut();
        state.update_pane_interaction(|interaction| {
            interaction.hovered_template_control_id.clear();
            interaction.hovered_template_dispatch_kind.clear();
            interaction.hovered_template_action_id.clear();
            interaction.hovered_template_value_text.clear();
            interaction.hovered_template_frame = FrameRect::default();
        });
    }
}
