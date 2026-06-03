use super::{
    HostClosePromptData, HostMenuStateData, HostNativeFloatingWindowSurfaceData,
    HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageData,
    HostWindowLayoutData, HostWindowSceneData, HostWindowShellData, TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::ModelRc;

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
    pub root_template_nodes: ModelRc<TemplatePaneNodeData>,
    /// Full-window componentized workbench surface nodes. These are separate
    /// from root template overlays so the shell cannot regress into a PNG-like overlay.
    pub workbench_window_nodes: ModelRc<TemplatePaneNodeData>,
    pub native_floating_surface_data: HostNativeFloatingWindowSurfaceData,
}
