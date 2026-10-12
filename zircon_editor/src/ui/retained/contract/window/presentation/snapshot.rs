use super::super::super::data::{FrameRect, HostWindowBootstrapData, HostWindowPresentationData};
use super::super::super::globals::HostContractState;
use super::super::UiHostWindow;

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
    state.presentation_generation().materialize()
}
