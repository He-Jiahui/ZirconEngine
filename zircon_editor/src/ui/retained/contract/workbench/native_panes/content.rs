use crate::ui::retained_host::hierarchy_pointer::current_hierarchy_row_metrics;

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, PaneData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use super::super::welcome;
use super::{assets, hierarchy, scrollbar};

pub(in crate::ui::retained_host::host_contract) fn draw_native_pane_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    let Some(effective_clip) = effective_native_clip(clip, frame.paint_clip()) else {
        return native_content_is_present(pane);
    };
    let clip = &effective_clip;
    match pane.kind.as_str() {
        "Welcome" => welcome::draw_welcome_native_content(frame, pane, body, clip),
        "Hierarchy" => {
            let content_present = native_content_is_present(pane);
            let viewport = hierarchy::hierarchy_viewport_frame(pane, body);
            let row_metrics = current_hierarchy_row_metrics();
            hierarchy::draw_hierarchy_rows(
                frame,
                pane,
                &viewport,
                clip,
                interaction,
                text_input_focus,
                row_metrics,
            );
            scrollbar::draw_hierarchy_scrollbar(
                frame,
                pane,
                &viewport,
                clip,
                interaction,
                row_metrics,
            );
            content_present
        }
        "Assets" => {
            let hover = assets::draw_activity_asset_tree_hover_overlay(
                frame,
                pane,
                body,
                clip,
                interaction,
            );
            let scrollbars =
                scrollbar::draw_activity_asset_scrollbars(frame, pane, body, clip, interaction);
            hover || scrollbars
        }
        "AssetBrowser" => {
            scrollbar::draw_browser_asset_scrollbars(frame, pane, body, clip, interaction)
        }
        _ => false,
    }
}

fn native_content_is_present(pane: &PaneData) -> bool {
    match pane.kind.as_str() {
        "Welcome" => pane.welcome.layout.has_nodes || !pane.welcome.title.is_empty(),
        "Hierarchy" => pane.hierarchy.hierarchy_nodes.row_count() > 0,
        _ => false,
    }
}

fn effective_native_clip(
    pane_clip: &FrameRect,
    paint_clip: Option<&FrameRect>,
) -> Option<FrameRect> {
    match paint_clip {
        Some(damage) => intersect(pane_clip, damage),
        None => Some(pane_clip.clone()),
    }
}

#[cfg(test)]
#[path = "tests/content.rs"]
mod tests;
