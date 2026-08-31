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
    let render_source_frame = match pane.kind.as_str() {
        "Assets" => pane.assets_activity.render_source_frame.as_ref(),
        "AssetBrowser" => pane.asset_browser.render_source_frame.as_ref(),
        _ => None,
    };
    frame.with_render_source_frame(render_source_frame, |frame| {
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
    })
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
            draw_template_nodes_with_transform(
                frame,
                nodes,
                origin,
                clip,
                text_input_focus,
                Some(&projector),
            );
            return true;
        }
    }
    if pane.kind.as_str() == "AssetBrowser" {
        if let Some(projector) = BrowserAssetContentProjector::new(nodes, origin, interaction) {
            draw_template_nodes_with_transform(
                frame,
                nodes,
                origin,
                clip,
                text_input_focus,
                Some(&projector),
            );
            return true;
        }
    }
    if pane.kind.as_str() == "Console" {
        if let Some(projector) = ConsoleOutputProjector::new(nodes, origin, interaction) {
            draw_template_nodes_with_transform(
                frame,
                nodes,
                origin,
                clip,
                text_input_focus,
                Some(&projector),
            );
            projector.draw_scrollbar(frame, clip);
            return true;
        }
    }
    draw_template_nodes(frame, nodes, origin, clip, text_input_focus);
    true
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ui::retained_host::primitives::VecModel;

    use super::*;

    #[test]
    fn template_content_remains_present_outside_damage() {
        let nodes = ModelRc::from(Rc::new(VecModel::from(vec![
            TemplatePaneNodeData::default(),
        ])));
        let mut pane = PaneData::default();
        pane.template_v2.nodes = nodes;
        let body = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 80.0,
        };
        let mut frame = HostRgbaFrame::recording_only(300, 300);
        frame.replace_paint_clip(Some(FrameRect {
            x: 200.0,
            y: 200.0,
            width: 20.0,
            height: 20.0,
        }));

        assert!(draw_pane_template_nodes(
            &mut frame,
            &pane,
            &body,
            &body,
            &HostPaneInteractionStateData::default(),
            None,
        ));
        assert!(frame.into_recorded_commands().is_empty());
    }
}
