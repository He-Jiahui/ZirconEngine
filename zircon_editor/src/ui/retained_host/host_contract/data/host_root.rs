use super::{
    HostAssetDeletionBlockerData, HostBottomDockSurfaceData, HostClosePromptData,
    HostDockOverflowMenuStateData, HostMenuStateData, HostNativeFloatingWindowSurfaceData,
    HostPageOverflowMenuStateData, HostPaneInteractionStateData, HostSideDockSurfaceData,
    HostTextInputFocusData, HostViewportImageSet, HostWindowLayoutData, HostWindowSceneData,
    HostWindowShellData, TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::ModelRc;
#[derive(Clone, Default)]
pub(crate) struct HostWindowPresentationData {
    pub host_shell: HostWindowShellData,
    pub host_layout: HostWindowLayoutData,
    pub host_scene_data: HostWindowSceneData,
    pub menu_state: HostMenuStateData,
    pub host_page_overflow_menu_state: HostPageOverflowMenuStateData,
    pub host_dock_overflow_menu_state: HostDockOverflowMenuStateData,
    pub asset_deletion_blocker: HostAssetDeletionBlockerData,
    pub close_prompt: HostClosePromptData,
    pub pane_interaction_state: HostPaneInteractionStateData,
    pub text_input_focus: HostTextInputFocusData,
    pub viewport_images: HostViewportImageSet,
    pub root_template_nodes: ModelRc<TemplatePaneNodeData>,
    /// Full-window componentized workbench surface nodes. These are separate
    /// from root template overlays so the shell cannot regress into a PNG-like overlay.
    pub workbench_window_nodes: ModelRc<TemplatePaneNodeData>,
    pub native_floating_surface_data: HostNativeFloatingWindowSurfaceData,
}

/// Geometry-bearing projection published independently from semantic pane authority.
#[derive(Clone)]
pub(crate) struct HostWindowGeometryPresentationData {
    pub host_shell: HostWindowShellData,
    pub host_layout: HostWindowLayoutData,
    pub host_scene_data: HostWindowSceneData,
    pub asset_deletion_blocker: HostAssetDeletionBlockerData,
    pub root_template_nodes: ModelRc<TemplatePaneNodeData>,
    pub workbench_window_nodes: ModelRc<TemplatePaneNodeData>,
    pub native_floating_surface_data: HostNativeFloatingWindowSurfaceData,
}

impl HostWindowGeometryPresentationData {
    pub(crate) fn from_presentation(presentation: &HostWindowPresentationData) -> Self {
        Self {
            host_shell: presentation.host_shell.clone(),
            host_layout: presentation.host_layout.clone(),
            host_scene_data: presentation.host_scene_data.clone(),
            asset_deletion_blocker: presentation.asset_deletion_blocker.clone(),
            root_template_nodes: presentation.root_template_nodes.clone(),
            workbench_window_nodes: presentation.workbench_window_nodes.clone(),
            native_floating_surface_data: presentation.native_floating_surface_data.clone(),
        }
    }

    pub(crate) fn apply_to(
        self,
        current: &HostWindowPresentationData,
    ) -> HostWindowPresentationData {
        let mut next = HostWindowPresentationData {
            host_shell: self.host_shell,
            host_layout: self.host_layout,
            host_scene_data: self.host_scene_data,
            menu_state: current.menu_state.clone(),
            host_page_overflow_menu_state: current.host_page_overflow_menu_state.clone(),
            host_dock_overflow_menu_state: current.host_dock_overflow_menu_state.clone(),
            asset_deletion_blocker: self.asset_deletion_blocker,
            close_prompt: current.close_prompt.clone(),
            pane_interaction_state: current.pane_interaction_state.clone(),
            text_input_focus: current.text_input_focus.clone(),
            viewport_images: current.viewport_images.clone(),
            root_template_nodes: self.root_template_nodes,
            workbench_window_nodes: self.workbench_window_nodes,
            native_floating_surface_data: self.native_floating_surface_data,
        };

        // Geometry publication cannot replace retained semantic pane products.
        next.host_scene_data.left_dock.pane = current.host_scene_data.left_dock.pane.clone();
        next.host_scene_data.document_dock.pane =
            current.host_scene_data.document_dock.pane.clone();
        next.host_scene_data.right_dock.pane = current.host_scene_data.right_dock.pane.clone();
        next.host_scene_data.bottom_dock.pane = current.host_scene_data.bottom_dock.pane.clone();
        next
    }
}

pub(crate) enum HostDockPresentationPatch {
    Left(HostSideDockSurfaceData),
    Right(HostSideDockSurfaceData),
    Bottom(HostBottomDockSurfaceData),
}
