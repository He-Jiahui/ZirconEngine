use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::template_component_family::template_component_family;
use super::super::surface_frame::hit_test_host_surface_frame;
use super::popup_rows::{hit_test_template_popup_rows, TemplatePopupRowHit};
use super::TemplateNodePointerHit;

pub(super) fn hit_test_template_nodes(
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
