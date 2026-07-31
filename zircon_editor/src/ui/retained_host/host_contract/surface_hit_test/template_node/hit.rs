use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::template_component_family::{
    TemplateComponentFamily, template_component_family,
};
use super::super::surface_frame::hit_test_host_surface_frame;
use super::TemplateNodePointerHit;
use super::popup_rows::{TemplatePopupRowHit, hit_test_template_popup_rows};

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
    let node = nodes.get(row)?;
    let frame = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    let component_family = template_component_family(node);
    let table_row = (component_family == Some(TemplateComponentFamily::Table))
        .then(|| table_row_hit(node, &frame, y))
        .flatten();
    Some(TemplateNodePointerHit {
        pane_id: SharedString::new(),
        control_id: node.control_id.clone(),
        action_id: node.action_id.clone(),
        binding_id: node.binding_id.clone(),
        dispatch_kind: node.dispatch_kind.clone(),
        component_role: node.component_role.clone(),
        component_family,
        value_text: node.value_text.clone(),
        edit_action_id: node.edit_action_id.clone(),
        commit_action_id: node.commit_action_id.clone(),
        disabled: node.disabled,
        frame,
        table_row_source_index: table_row.as_ref().map(|row| row.source_index),
        table_row_identity_kind: table_row
            .as_ref()
            .map(|row| row.identity_kind.clone())
            .unwrap_or_default(),
        table_row_identity_text: table_row
            .as_ref()
            .map(|row| row.identity_text.clone())
            .unwrap_or_default(),
    })
}

fn table_row_hit(
    node: &super::super::super::data::TemplatePaneNodeData,
    frame: &FrameRect,
    y: f32,
) -> Option<super::super::super::data::TemplatePaneCollectionRowData> {
    let row_count = node.collection_rows.row_count();
    if row_count == 0 || frame.height <= 0.0 {
        return None;
    }
    let row_height = if node.virtualization_enabled && node.virtualization_item_extent > 0.0 {
        node.virtualization_item_extent
    } else {
        frame.height / row_count as f32
    };
    if row_height <= 0.0 {
        return None;
    }
    let row_index = ((y - frame.y) / row_height).floor() as isize;
    usize::try_from(row_index)
        .ok()
        .and_then(|index| node.collection_rows.row_data(index))
        .filter(|row| !row.row_identity_field.is_empty())
}
