use zircon_runtime::ui::{
    layout::{
        Anchor, AxisConstraint, BoxConstraints, Pivot, Position, StretchMode, UiFrame, UiSize,
    },
    surface::UiSurface,
};

use crate::ui::slint_host::callback_dispatch::constants::{
    BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID, UI_HOST_WINDOW_CONTROL_ID,
};
use crate::ui::template_runtime::{EditorUiHostRuntime, SlintUiHostProjection, SlintUiProjection};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;

use super::bridge::HOST_PAGE_STRIP_CONTROL_ID;
use super::error::BuiltinWorkbenchTemplateBridgeError;

pub(super) fn build_builtin_workbench_host_projection(
    runtime: &EditorUiHostRuntime,
    projection: &SlintUiProjection,
    shell_size: UiSize,
) -> Result<SlintUiHostProjection, BuiltinWorkbenchTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    apply_builtin_workbench_host_strip_layout(&mut surface);
    surface.compute_layout(shell_size)?;
    Ok(runtime.build_slint_host_projection_with_surface(projection, &surface)?)
}

fn apply_builtin_workbench_host_strip_layout(surface: &mut UiSurface) {
    let Some(shell_frame) = surface_control_frame(surface, UI_HOST_WINDOW_CONTROL_ID) else {
        return;
    };

    let metrics = WorkbenchChromeMetrics::default();
    apply_fixed_control_frame(
        surface,
        HOST_PAGE_STRIP_CONTROL_ID,
        UiFrame::new(
            shell_frame.x,
            shell_frame.y + metrics.top_bar_height + metrics.separator_thickness,
            shell_frame.width.max(0.0),
            metrics.host_bar_height.max(0.0),
        ),
    );
}

fn surface_control_frame(surface: &UiSurface, control_id: &str) -> Option<UiFrame> {
    let node_id = surface_control_node_id(surface, control_id)?;
    surface
        .tree
        .node(node_id)
        .map(|node| node.layout_cache.frame)
}

fn surface_control_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> Option<zircon_runtime::ui::event_ui::UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}

fn apply_fixed_control_frame(surface: &mut UiSurface, control_id: &str, frame: UiFrame) {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return;
    };
    let Some(node) = surface.tree.node_mut(node_id) else {
        return;
    };

    node.anchor = Anchor::default();
    node.pivot = Pivot::default();
    node.position = Position::new(frame.x, frame.y);
    node.constraints = fixed_box_constraints(frame.width, frame.height);
    node.state_flags.visible = frame.width > f32::EPSILON && frame.height > f32::EPSILON;
}

fn fixed_box_constraints(width: f32, height: f32) -> BoxConstraints {
    BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
    }
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size.max(0.0),
        max: size.max(0.0),
        preferred: size.max(0.0),
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
