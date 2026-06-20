use super::super::super::data::{FrameRect, HostPaneInteractionStateData, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::welcome;
use super::{assets, hierarchy};

pub(in crate::ui::retained_host::host_contract) fn draw_native_pane_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    match pane.kind.as_str() {
        "Welcome" => welcome::draw_welcome_native_content(frame, pane, body, clip),
        "Hierarchy" => hierarchy::draw_hierarchy_rows(frame, pane, body, clip, interaction),
        "Assets" => {
            assets::draw_activity_asset_tree_hover_overlay(frame, pane, body, clip, interaction)
        }
        "AssetBrowser" => {
            assets::draw_browser_asset_tree_hover_overlay(frame, pane, body, clip, interaction)
        }
        _ => false,
    }
}
