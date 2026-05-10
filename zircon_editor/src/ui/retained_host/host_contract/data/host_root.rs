use super::{
    HostClosePromptData, HostMenuStateData, HostNativeFloatingWindowSurfaceData,
    HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageData,
    HostWindowLayoutData, HostWindowSceneData, HostWindowShellData,
};

#[derive(Clone, Default)]
pub(crate) struct HostWindowPresentationData {
    pub host_shell: HostWindowShellData,
    pub host_layout: HostWindowLayoutData,
    pub host_scene_data: HostWindowSceneData,
    pub menu_state: HostMenuStateData,
    pub close_prompt: HostClosePromptData,
    pub pane_interaction_state: HostPaneInteractionStateData,
    pub text_input_focus: HostTextInputFocusData,
    pub viewport_image: Option<HostViewportImageData>,
    pub native_floating_surface_data: HostNativeFloatingWindowSurfaceData,
}
