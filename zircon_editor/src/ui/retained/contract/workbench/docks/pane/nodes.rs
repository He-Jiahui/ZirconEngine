#[path = "../../../docks/pane/nodes/asset_content/mod.rs"]
mod asset_content;
#[path = "../../../docks/pane/nodes/console_output.rs"]
mod console_output;
#[path = "nodes/selection.rs"]
mod selection;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, PaneData, TemplatePaneNodeData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{
    draw_runtime_surface_frame, draw_template_nodes, draw_template_nodes_with_transform,
    has_template_nodes,
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
    let shell_drawn = frame.with_render_source_frame(render_source_frame, |frame| {
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
    });
    let preview_drawn = if pane.kind.as_str() == "UiAssetEditor" {
        let canvas = &pane.ui_asset.preview.canvas;
        if let (Some(source), Some(transform)) = (
            canvas.source_surface_frame.as_ref(),
            canvas.transform(&pane.ui_asset.designer_canvas_panel_node, body),
        ) {
            if let Some(canvas_clip) =
                super::super::super::super::paint_geometry::intersect(&transform.clip, clip)
            {
                draw_runtime_surface_frame(
                    frame,
                    source,
                    transform.origin_x,
                    transform.origin_y,
                    transform.scale,
                    &canvas_clip,
                )
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    shell_drawn || preview_drawn
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
#[path = "tests/nodes.rs"]
mod tests;
