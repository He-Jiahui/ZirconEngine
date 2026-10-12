use super::{intersect_frames, reaches_ancestor, HostExtensionWorkspacePaintIndex};
use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
    authored_level_viewport, authored_panes, componentized_workbench_chrome_regions,
    module_tools_popups, ModuleToolsPopup,
};
use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ComponentizedHitOwnership {
    top_chrome: FrameRect,
    status_bar: FrameRect,
    extension_workspace: Option<(usize, FrameRect)>,
    ordinary_panes: Vec<(usize, FrameRect)>,
    module_tools_popups: Vec<ModuleToolsPopup>,
}

impl ComponentizedHitOwnership {
    pub(super) fn module_tools_popup_contains_point(&self, row: usize, x: f32, y: f32) -> bool {
        self.module_tools_popups.iter().any(|popup| {
            popup.row == row
                && frame_contains_point(&popup.frame, x, y)
                && frame_contains_point(&popup.clip, x, y)
        })
    }

    pub(super) fn module_tools_popup_clip(&self, row: usize) -> Option<&FrameRect> {
        self.module_tools_popups
            .iter()
            .find(|popup| popup.row == row)
            .map(|popup| &popup.clip)
    }

    pub(super) fn row_contains_painted_point(
        &self,
        row: usize,
        frame: &FrameRect,
        x: f32,
        y: f32,
        parent_rows: &[Option<usize>],
    ) -> bool {
        (self
            .extension_workspace
            .as_ref()
            .is_some_and(|(root_row, clip)| {
                reaches_ancestor(row, *root_row, parent_rows) && frame_contains_point(clip, x, y)
            })
            || self.ordinary_panes.iter().any(|(root_row, clip)| {
                reaches_ancestor(row, *root_row, parent_rows) && frame_contains_point(clip, x, y)
            })
            || frame_contains_point(&self.top_chrome, x, y)
            || frame_contains_point(&self.status_bar, x, y))
            && frame_contains_point(frame, x, y)
    }
}

pub(super) fn componentized_hit_ownership(
    presentation: &HostWindowPresentationData,
    frame_bounds: Option<&FrameRect>,
    extension_workspace: Option<&HostExtensionWorkspacePaintIndex>,
) -> Option<ComponentizedHitOwnership> {
    let frame_bounds = frame_bounds?;
    let chrome = componentized_workbench_chrome_regions(&presentation.host_layout, frame_bounds)?;
    let extension_workspace = extension_workspace.and_then(|workspace| {
        intersect_frames(&workspace.host_frame, frame_bounds).map(|clip| (workspace.root_row, clip))
    });
    Some(ComponentizedHitOwnership {
        top_chrome: chrome.top_chrome,
        status_bar: chrome.status_bar,
        module_tools_popups: module_tools_popups(presentation, frame_bounds),
        extension_workspace,
        ordinary_panes: authored_panes(presentation)
            .into_iter()
            .filter(|pane| {
                pane.root.control_id.as_str() != "WorkbenchMainBandViewportPanel"
                    || pane.module_workspace.is_some()
                    || presentation
                        .host_scene_data
                        .document_surfaces()
                        .iter()
                        .any(|document| {
                            authored_level_viewport(presentation, document)
                                .is_some_and(|viewport| viewport.root.node_id == pane.root.node_id)
                        })
            })
            .map(|pane| (pane.root_row, pane.frame))
            .collect(),
    })
}

fn frame_contains_point(frame: &FrameRect, x: f32, y: f32) -> bool {
    frame.width > 0.0
        && frame.height > 0.0
        && x >= frame.x
        && y >= frame.y
        && x < frame.x + frame.width
        && y < frame.y + frame.height
}
