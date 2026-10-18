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
    /// Full arranged observations; only the exact immutable presentation may publish them.
    pub(crate) normal_arranged_inventory: Option<(
        super::HostPresentationGenerationCursor,
        std::sync::Arc<crate::ui::template_runtime::NormalPresentInventory>,
    )>,
    /// Immutable capture metadata; stale stamps are omitted at the successful present boundary.
    pub(crate) normal_workbench_business: Option<(
        super::HostPresentationGenerationCursor,
        std::sync::Arc<crate::ui::retained_host::ui::NormalWorkbenchBusinessSnapshot>,
    )>,
    /// Prepared readonly toolbar metadata; associated only at successful present.
    pub(crate) normal_viewport_toolbar: Option<(
        super::HostPresentationGenerationCursor,
        std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
        std::sync::Arc<serde_json::Value>,
    )>,
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
            normal_arranged_inventory: None,
            normal_workbench_business: None,
            normal_viewport_toolbar: None,
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

impl HostWindowPresentationData {
    pub(crate) fn normal_inventory_for_present(
        &self,
        cursor: super::HostPresentationGenerationCursor,
        successful_present: bool,
    ) -> Option<serde_json::Value> {
        let (stamp, inventory) = self.normal_arranged_inventory.as_ref()?;
        (successful_present && *stamp == cursor).then(|| inventory.observation())
    }
}

pub(crate) enum HostDockPresentationPatch {
    Left(HostSideDockSurfaceData),
    Right(HostSideDockSurfaceData),
    Bottom(HostBottomDockSurfaceData),
}

#[cfg(test)]
mod normal_inventory_tests {
    use super::super::HostPresentationGenerationCursor;
    use super::*;
    use crate::ui::template_runtime::NormalPresentInventory;
    use std::sync::Arc;

    fn cursor(values: [u64; 7]) -> HostPresentationGenerationCursor {
        HostPresentationGenerationCursor::new_with_geometry(
            values[0], values[1], values[2], values[3], values[4], values[5], values[6],
        )
    }

    #[test]
    fn normal_inventory_publication_requires_a_successful_present_and_the_full_cursor() {
        let generations = [1, 2, 3, 4, 5, 6, 7];
        let stamp = cursor(generations);
        let presentation = HostWindowPresentationData {
            normal_arranged_inventory: Some((
                stamp,
                Arc::new(NormalPresentInventory::new(Vec::new())),
            )),
            ..Default::default()
        };
        assert!(presentation
            .normal_inventory_for_present(stamp, true)
            .is_some());
        assert!(presentation
            .normal_inventory_for_present(stamp, false)
            .is_none());
        for index in 0..7 {
            let mut changed = generations;
            changed[index] += 1;
            assert!(
                presentation
                    .normal_inventory_for_present(cursor(changed), true)
                    .is_none(),
                "cursor field {index}"
            );
        }
    }

    #[test]
    fn geometry_publication_clears_inventory_without_mutating_the_old_snapshot() {
        let stamp = cursor([1, 2, 3, 4, 5, 6, 7]);
        let presentation = Arc::new(HostWindowPresentationData {
            normal_arranged_inventory: Some((
                stamp,
                Arc::new(NormalPresentInventory::new(Vec::new())),
            )),
            ..Default::default()
        });
        let geometry = HostWindowGeometryPresentationData::from_presentation(&presentation);
        let next = geometry.apply_to(&presentation);
        assert!(next.normal_arranged_inventory.is_none());
        assert!(presentation
            .normal_inventory_for_present(stamp, true)
            .is_some());
    }
}
