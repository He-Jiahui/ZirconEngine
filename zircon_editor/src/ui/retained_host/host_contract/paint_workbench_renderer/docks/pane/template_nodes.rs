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

// 按 pane 类型选择渲染源框与模板节点；专用投影器随后消费代次元数据维持滚动与裁剪，其他节点走通用绘制。
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
#[path = "tests/template_nodes.rs"]
mod tests;
