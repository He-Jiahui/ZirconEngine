use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::{layout::UiPoint, surface::UiSurfaceFrame};

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{
    FrameRect, HostSideDockSurfaceData, PaneData, TemplatePaneNodeData,
};
use super::super::UiProfileNamedFrame;
use super::frame_math::{
    frame_rect_center_point, intersect_frames, intersect_profile_frame, is_visible_frame,
    is_visible_profile_frame, push_named_frame, push_named_profile_frame, translated,
    translated_template_frame,
};

pub(in crate::ui::retained_host::host_contract) fn collect_activity_rail_buttons(
    surface: &str,
    dock: &HostSideDockSurfaceData,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    if dock.rail_width_px <= 0.0 || !is_visible_frame(&dock.region_frame) {
        return;
    }
    let rail_x = if dock.rail_before_panel {
        dock.region_frame.x
    } else {
        dock.region_frame.x + (dock.region_frame.width - dock.rail_width_px).max(0.0)
    };
    let rail = FrameRect {
        x: rail_x,
        y: dock.region_frame.y,
        width: dock.rail_width_px.min(dock.region_frame.width.max(0.0)),
        height: dock.region_frame.height,
    };
    for row in 0..dock.rail_button_frames.row_count() {
        let Some(button) = dock.rail_button_frames.row_data(row) else {
            continue;
        };
        let frame = translated(&button.frame, rail.x, rail.y);
        push_named_frame(
            out,
            format!("activity_rail.{surface}.{}", button.control_id).as_str(),
            "activity_rail_button",
            surface,
            frame,
            None,
        );
    }
}

pub(in crate::ui::retained_host::host_contract) fn collect_pane_profile_frames(
    surface: &str,
    pane: &PaneData,
    content: &FrameRect,
    viewport_toolbar_controls: &mut Vec<UiProfileNamedFrame>,
    template_controls: &mut Vec<UiProfileNamedFrame>,
) {
    if !is_visible_frame(content) {
        return;
    }
    let mut body = content.clone();
    if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar_height = 28.0_f32.min(content.height);
        let toolbar = FrameRect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: toolbar_height,
        };
        collect_surface_frame_controls(
            "viewport_toolbar_control",
            surface,
            &toolbar,
            pane.viewport.toolbar_surface_frame.as_ref(),
            viewport_toolbar_controls,
        );
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    collect_template_node_controls(surface, pane, &body, template_controls);
}

fn collect_template_node_controls(
    surface: &str,
    pane: &PaneData,
    body: &FrameRect,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let Some(nodes) = pane_template_nodes(pane) else {
        return;
    };
    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !is_dispatchable_template_node(&node) {
            continue;
        }
        let frame = translated_template_frame(&node.frame, body.x, body.y);
        let clip = node
            .has_clip_frame
            .then(|| translated_template_frame(&node.clip_frame, body.x, body.y).into());
        let effective_frame = if let Some(clip_frame) = clip.as_ref() {
            let Some(frame) = intersect_profile_frame(&frame, clip_frame) else {
                continue;
            };
            frame
        } else {
            frame.clone().into()
        };
        if !is_visible_profile_frame(&effective_frame) {
            continue;
        }
        push_named_profile_frame(
            out,
            format!("template.{surface}.{}", node.control_id).as_str(),
            "template_control",
            surface,
            effective_frame,
            clip,
        );
    }
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn collect_surface_frame_controls(
    kind: &str,
    surface: &str,
    origin: &FrameRect,
    surface_frame: Option<&UiSurfaceFrame>,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    collect_surface_frame_controls_impl(kind, surface, origin, surface_frame, out);
}

#[cfg(not(test))]
fn collect_surface_frame_controls(
    kind: &str,
    surface: &str,
    origin: &FrameRect,
    surface_frame: Option<&UiSurfaceFrame>,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    collect_surface_frame_controls_impl(kind, surface, origin, surface_frame, out);
}

fn collect_surface_frame_controls_impl(
    kind: &str,
    surface: &str,
    origin: &FrameRect,
    surface_frame: Option<&UiSurfaceFrame>,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let Some(surface_frame) = surface_frame else {
        return;
    };
    for node in &surface_frame.arranged_tree.nodes {
        if !node.supports_pointer() {
            continue;
        }
        let Some(control_id) = node.control_id.as_deref() else {
            continue;
        };
        let frame = FrameRect {
            x: origin.x + node.frame.x,
            y: origin.y + node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let clip = FrameRect {
            x: origin.x + node.clip_frame.x,
            y: origin.y + node.clip_frame.y,
            width: node.clip_frame.width,
            height: node.clip_frame.height,
        };
        let Some(effective_frame) = intersect_frames(&frame, &clip) else {
            continue;
        };
        let center = frame_rect_center_point(&effective_frame);
        let local_center = UiPoint::new(center.x - origin.x, center.y - origin.y);
        let route_is_top_hit = hit_test_surface_frame(surface_frame, local_center)
            .top_hit
            .and_then(|node_id| surface_frame.arranged_tree.get(node_id))
            .and_then(|hit_node| hit_node.control_id.as_deref())
            .is_some_and(|hit_control_id| hit_control_id == control_id);
        if !route_is_top_hit {
            continue;
        }
        push_named_profile_frame(
            out,
            format!("{kind}.{surface}.{control_id}").as_str(),
            kind,
            surface,
            effective_frame.into(),
            Some(clip.into()),
        );
    }
}

fn pane_template_nodes(pane: &PaneData) -> Option<&ModelRc<TemplatePaneNodeData>> {
    match pane.kind.as_str() {
        "Hierarchy" => Some(&pane.hierarchy.nodes),
        "Inspector" => Some(&pane.inspector.nodes),
        "Console" => Some(&pane.console.nodes),
        "Assets" => Some(&pane.assets_activity.nodes),
        "AssetBrowser" => Some(&pane.asset_browser.nodes),
        "Welcome" => Some(&pane.welcome.nodes),
        "Project" | "UiComponentShowcase" => Some(&pane.project_overview.nodes),
        "RuntimeDiagnostics" => Some(&pane.runtime_diagnostics.nodes),
        "PerformanceTimeline" => Some(&pane.performance_timeline.nodes),
        "ModulePlugins" => Some(&pane.module_plugins.nodes),
        "BuildExport" => Some(&pane.build_export.nodes),
        "GeneratedBottom" => Some(&pane.generated_bottom.nodes),
        "UiAssetEditor" => Some(&pane.ui_asset.nodes),
        "AnimationSequenceEditor" | "AnimationGraphEditor" => Some(&pane.animation.nodes),
        _ => None,
    }
}

fn is_dispatchable_template_node(node: &TemplatePaneNodeData) -> bool {
    !node.disabled
        && !node.control_id.is_empty()
        && (!node.action_id.is_empty()
            || !node.binding_id.is_empty()
            || !node.dispatch_kind.is_empty()
            || !node.edit_action_id.is_empty()
            || !node.commit_action_id.is_empty()
            || matches!(node.component_role.as_str(), "input-field" | "number-field"))
}

pub(in crate::ui::retained_host::host_contract) fn side_dock_content_frame(
    dock: &HostSideDockSurfaceData,
) -> FrameRect {
    let panel_x = if dock.rail_before_panel {
        dock.region_frame.x + dock.rail_width_px
    } else {
        dock.region_frame.x
    };
    translated(&dock.content_frame, panel_x, dock.region_frame.y)
}

pub(in crate::ui::retained_host::host_contract) fn floating_window_content_frame(
    frame: &FrameRect,
    header: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: frame.x + 1.0,
        y: frame.y + header.height.max(0.0) + 1.0,
        width: (frame.width - 2.0).max(0.0),
        height: (frame.height - header.height.max(0.0) - 2.0).max(0.0),
    }
}
