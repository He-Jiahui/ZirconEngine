use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

#[derive(Debug, Default, PartialEq)]
pub(super) struct ToolbarControlSnapshot {
    pub(super) toolbar: Option<ViewTemplateFrameData>,
    pub(super) import_panel: Option<ViewTemplateFrameData>,
    pub(super) locate_width: Option<f32>,
    pub(super) filter_width: Option<f32>,
    pub(super) list_width: Option<f32>,
    pub(super) thumb_width: Option<f32>,
    pub(super) import_button_width: Option<f32>,
}

impl ToolbarControlSnapshot {
    pub(super) fn from_nodes(nodes: &[ViewTemplateNodeData]) -> Self {
        let mut snapshot = Self::default();
        let mut remaining = 7;
        for node in nodes {
            let found = match node.control_id.as_str() {
                "AssetBrowserToolbarPanel" if snapshot.toolbar.is_none() => {
                    snapshot.toolbar = Some(node.frame.clone());
                    true
                }
                "AssetBrowserImportPanel" if snapshot.import_panel.is_none() => {
                    snapshot.import_panel = Some(node.frame.clone());
                    true
                }
                "LocateSelectedAsset" if snapshot.locate_width.is_none() => {
                    snapshot.locate_width = Some(node.frame.width);
                    true
                }
                "AssetBrowserKindFilterDropdown" if snapshot.filter_width.is_none() => {
                    snapshot.filter_width = Some(node.frame.width);
                    true
                }
                "AssetBrowserViewModeListButton" if snapshot.list_width.is_none() => {
                    snapshot.list_width = Some(node.frame.width);
                    true
                }
                "AssetBrowserViewModeThumbButton" if snapshot.thumb_width.is_none() => {
                    snapshot.thumb_width = Some(node.frame.width);
                    true
                }
                "ImportModel" if snapshot.import_button_width.is_none() => {
                    snapshot.import_button_width = Some(node.frame.width);
                    true
                }
                _ => false,
            };
            if found {
                remaining -= 1;
                if remaining == 0 {
                    break;
                }
            }
        }
        snapshot
    }
}

pub(super) fn width_or(observed: Option<f32>, fallback: f32) -> f32 {
    observed.filter(|width| *width > 0.0).unwrap_or(fallback)
}
