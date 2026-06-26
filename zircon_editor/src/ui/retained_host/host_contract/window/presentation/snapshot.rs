use super::super::super::data::{FrameRect, HostWindowBootstrapData, HostWindowPresentationData};
use super::super::super::globals::HostContractState;
use super::super::{template_hover, UiHostWindow};

impl UiHostWindow {
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
    presentation.host_page_overflow_menu_state = state.host_page_overflow_menu_state.clone();
    presentation.pane_interaction_state = state.pane_interaction_state.clone();
    presentation.text_input_focus = state.text_input_focus.clone();
    presentation.viewport_image = state.viewport_image.clone();
    template_hover::apply_template_hover_to_presentation(
        &mut presentation,
        &state.pane_interaction_state,
    );
    presentation
}
