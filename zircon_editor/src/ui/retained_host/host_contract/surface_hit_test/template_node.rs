mod pane_nodes;
mod popup_rows;
mod surface_frame_builder;

use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use zircon_runtime_interface::ui::{layout::UiSize, surface::UiSurfaceFrame};

use super::super::data::{FrameRect, HostWindowPresentationData, PaneData, TemplatePaneNodeData};
use super::super::template_component_family::{template_component_family, TemplateComponentFamily};
use super::super::template_geometry::template_nodes_bounds;
use super::surface_frame::hit_test_host_surface_frame;
use pane_nodes::pane_template_nodes;
use popup_rows::{hit_test_template_popup_rows, TemplatePopupRowHit};
use surface_frame_builder::{build_template_surface_frame, template_nodes_surface_frame};

#[derive(Clone)]
pub(crate) struct TemplateNodePointerHit {
    pub(crate) control_id: SharedString,
    pub(crate) action_id: SharedString,
    pub(crate) binding_id: SharedString,
    pub(crate) dispatch_kind: SharedString,
    pub(crate) component_role: SharedString,
    pub(crate) component_family: Option<TemplateComponentFamily>,
    pub(crate) value_text: SharedString,
    pub(crate) edit_action_id: SharedString,
    pub(crate) commit_action_id: SharedString,
    pub(crate) frame: FrameRect,
}

pub(crate) fn hit_test_pane_template_node(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let nodes = pane_template_nodes(pane)?;
    let surface_frame = pane.body_surface_frame.as_ref()?;
    hit_test_template_nodes(nodes, surface_frame, body, x, y)
}

pub(crate) fn hit_test_workbench_window_template_node(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let nodes = &presentation.workbench_window_nodes;
    let bounds = template_nodes_bounds(nodes)?;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: bounds.width.max(bounds.x + bounds.width).max(1.0),
        height: bounds.height.max(bounds.y + bounds.height).max(1.0),
    };
    let surface_frame =
        template_nodes_surface_frame(nodes, UiSize::new(origin.width, origin.height));
    hit_test_template_nodes(nodes, &surface_frame, &origin, x, y)
}

pub(crate) fn build_pane_template_surface_frame(
    pane: &PaneData,
    surface_size: UiSize,
) -> Option<UiSurfaceFrame> {
    build_template_surface_frame(pane_template_nodes(pane)?, surface_size)
}

fn hit_test_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_frame: &UiSurfaceFrame,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    match hit_test_template_popup_rows(nodes, origin, x, y) {
        Some(TemplatePopupRowHit::Hit(hit)) => return Some(hit),
        Some(TemplatePopupRowHit::Blocked) => return None,
        None => {}
    }

    let hit = hit_test_host_surface_frame(surface_frame, origin, x, y)?;
    let row = hit.node_id.0.checked_sub(2)? as usize;
    let node = nodes.row_data(row)?;
    let frame = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    let component_family = template_component_family(&node);
    Some(TemplateNodePointerHit {
        control_id: node.control_id,
        action_id: node.action_id,
        binding_id: node.binding_id,
        dispatch_kind: node.dispatch_kind,
        component_role: node.component_role,
        component_family,
        value_text: node.value_text,
        edit_action_id: node.edit_action_id,
        commit_action_id: node.commit_action_id,
        frame,
    })
}

#[cfg(test)]
#[path = "template_node_tests.rs"]
mod tests;
