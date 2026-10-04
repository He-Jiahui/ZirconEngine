use super::super::super::data::FrameRect;
use super::super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn set_template_button_pointer_state(
        &self,
        control_id: Option<&str>,
        frame: Option<&FrameRect>,
        pressed: bool,
    ) -> Option<FrameRect> {
        let mut state = self.state.borrow_mut();
        let before = state.pane_interaction_state.as_ref().clone();
        state.update_pane_interaction(|interaction| {
            if pressed {
                interaction.focused_template_control_id = control_id.unwrap_or_default().into();
                interaction.pressed_template_control_id = control_id.unwrap_or_default().into();
                interaction.template_button_frame = frame.cloned().unwrap_or_default();
                // Pointer focus is semantic and does not imitate keyboard focus-visible.
                interaction.template_focus_visible = false;
            } else {
                interaction.pressed_template_control_id.clear();
            }
        });
        let after = state.pane_interaction_state.as_ref();
        if before == *after {
            return None;
        }
        let old = &before.template_button_frame;
        let new = &after.template_button_frame;
        if old.width <= 0.0 || old.height <= 0.0 {
            return (new.width > 0.0 && new.height > 0.0).then(|| new.clone());
        }
        if new.width <= 0.0 || new.height <= 0.0 {
            return Some(old.clone());
        }
        let x = old.x.min(new.x);
        let y = old.y.min(new.y);
        Some(FrameRect {
            x,
            y,
            width: old.right().max(new.right()) - x,
            height: old.bottom().max(new.bottom()) - y,
        })
    }

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
