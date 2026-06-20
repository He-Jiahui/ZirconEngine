use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageData,
    PaneData, TemplatePaneNodeData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::{draw_rect, draw_text_bars_clipped};
use super::super::super::paint_template_nodes::{draw_template_nodes, has_template_nodes};
use super::super::{first_non_empty, native_panes, MUTED_TEXT, PANE_EMPTY, VIEWPORT_PANEL};
use super::viewport_toolbar;

pub(in crate::ui::retained_host::host_contract) fn draw_pane(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    content: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(content) {
        return;
    }
    let pane_color = match pane.kind.as_str() {
        "Scene" | "Game" => VIEWPORT_PANEL,
        _ => PANE_EMPTY,
    };
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_background");
        draw_rect(frame, content.clone(), pane_color);
    }

    let body = if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar = FrameRect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: 28.0_f32.min(content.height),
        };
        {
            zircon_runtime::profile_scope!(
                "editor",
                "host_painter",
                "painter_pane_viewport_toolbar"
            );
            viewport_toolbar::draw_viewport_toolbar(frame, pane, &toolbar, content);
        }
        FrameRect {
            x: content.x,
            y: content.y + toolbar.height,
            width: content.width,
            height: (content.height - toolbar.height).max(0.0),
        }
    } else {
        content.clone()
    };

    let painted_viewport = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_viewport_image");
        native_panes::draw_viewport_image(frame, pane, &body, content, viewport_image)
    };
    let painted_nodes = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_template_nodes");
        draw_pane_template_nodes(frame, pane, &body, content, text_input_focus)
    };
    let painted_native = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_native_content");
        native_panes::draw_native_pane_content(frame, pane, &body, content, interaction)
    };
    let painted_debug_overlay = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_debug_overlay");
        native_panes::draw_pane_debug_overlay(frame, pane, &body, content)
    };
    if !painted_viewport && !painted_nodes && !painted_native && !painted_debug_overlay {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_fallback");
        draw_pane_fallback(frame, pane, &body, content);
    }
}

fn draw_pane_template_nodes(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    match pane.kind.as_str() {
        "Hierarchy" => draw_if_present(frame, &pane.hierarchy.nodes, body, clip, text_input_focus),
        "Inspector" => draw_if_present(frame, &pane.inspector.nodes, body, clip, text_input_focus),
        "Console" => draw_if_present(frame, &pane.console.nodes, body, clip, text_input_focus),
        "Assets" => draw_if_present(
            frame,
            &pane.assets_activity.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "AssetBrowser" => draw_if_present(
            frame,
            &pane.asset_browser.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "Welcome" => draw_if_present(frame, &pane.welcome.nodes, body, clip, text_input_focus),
        "Project" | "UiComponentShowcase" => draw_if_present(
            frame,
            &pane.project_overview.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "RuntimeDiagnostics" => draw_if_present(
            frame,
            &pane.runtime_diagnostics.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "PerformanceTimeline" => draw_if_present(
            frame,
            &pane.performance_timeline.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "ModulePlugins" => draw_if_present(
            frame,
            &pane.module_plugins.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "BuildExport" => draw_if_present(
            frame,
            &pane.build_export.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "GeneratedBottom" => draw_if_present(
            frame,
            &pane.generated_bottom.nodes,
            body,
            clip,
            text_input_focus,
        ),
        "UiAssetEditor" => {
            draw_if_present(frame, &pane.ui_asset.nodes, body, clip, text_input_focus)
        }
        "AnimationSequenceEditor" | "AnimationGraphEditor" => {
            draw_if_present(frame, &pane.animation.nodes, body, clip, text_input_focus)
        }
        _ => false,
    }
}

fn draw_if_present(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    has_template_nodes(nodes) && draw_template_nodes(frame, nodes, origin, clip, text_input_focus)
}

fn draw_pane_fallback(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) {
    let label = first_non_empty(&[
        pane.title.as_str(),
        pane.kind.as_str(),
        pane.subtitle.as_str(),
        pane.info.as_str(),
    ]);
    draw_text_bars_clipped(
        frame,
        body.x + 12.0,
        body.y + 16.0,
        label,
        Some(clip),
        MUTED_TEXT,
    );
}
