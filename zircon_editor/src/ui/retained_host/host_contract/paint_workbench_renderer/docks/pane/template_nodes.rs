mod asset_content;
mod console_output;
mod selection;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, PaneData, TemplatePaneNodeData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{
    draw_template_nodes, draw_template_nodes_with_transform, has_template_nodes,
};

use asset_content::{ActivityAssetContentProjector, BrowserAssetContentProjector};
use console_output::ConsoleOutputProjector;
use selection::select_pane_template_nodes;

pub(super) fn draw_pane_template_nodes(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    select_pane_template_nodes(pane)
        .map(|nodes| {
            draw_if_present(
                frame,
                pane,
                nodes,
                body,
                clip,
                interaction,
                text_input_focus,
            )
        })
        .unwrap_or(false)
}

fn draw_if_present(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    if !has_template_nodes(nodes) {
        return false;
    }
    if pane.kind.as_str() == "Assets" {
        if let Some(projector) = ActivityAssetContentProjector::new(nodes, origin, interaction) {
            return draw_template_nodes_with_transform(
                frame,
                nodes,
                origin,
                clip,
                text_input_focus,
                Some(&projector),
            );
        }
    }
    if pane.kind.as_str() == "AssetBrowser" {
        if let Some(projector) = BrowserAssetContentProjector::new(nodes, origin, interaction) {
            return draw_template_nodes_with_transform(
                frame,
                nodes,
                origin,
                clip,
                text_input_focus,
                Some(&projector),
            );
        }
    }
    if pane.kind.as_str() == "Console" {
        if let Some(projector) = ConsoleOutputProjector::new(nodes, origin, interaction) {
            let nodes_painted = draw_template_nodes_with_transform(
                frame,
                nodes,
                origin,
                clip,
                text_input_focus,
                Some(&projector),
            );
            return projector.draw_scrollbar(frame, clip) || nodes_painted;
        }
    }
    draw_template_nodes(frame, nodes, origin, clip, text_input_focus)
}
