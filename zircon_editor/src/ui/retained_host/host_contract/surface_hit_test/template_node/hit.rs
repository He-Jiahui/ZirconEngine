use crate::ui::retained_host::console_output::ConsoleOutputPaintMetadata;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::frame_geometry::contains_point;
use super::super::super::template_component_family::{
    template_component_family, TemplateComponentFamily,
};
use super::super::surface_frame::hit_test_host_surface_frame;
use super::popup_rows::{
    hit_test_template_popup_node, hit_test_template_popup_rows, TemplatePopupRowHit,
};
use super::surface_frame_builder::is_dispatchable;
use super::HostPaneTemplateHitIndex;
use super::HostWorkbenchHitIndex;
use super::TemplateNodePointerHit;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) fn hit_test_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_frame: &UiSurfaceFrame,
    origin: &FrameRect,
    x: f32,
    y: f32,
    popup_index: Option<&HostPaneTemplateHitIndex>,
) -> Option<TemplateNodePointerHit> {
    match hit_test_template_popup_rows(nodes, popup_index, origin, x, y) {
        Some(TemplatePopupRowHit::Hit(hit)) => return Some(hit),
        Some(TemplatePopupRowHit::Blocked) => return None,
        None => {}
    }

    let hit = hit_test_host_surface_frame(surface_frame, origin, x, y)?;
    let row = hit.node_id.0.checked_sub(2)? as usize;
    let node = nodes.get(row)?;
    Some(template_node_pointer_hit(node, origin, y))
}

pub(super) fn hit_test_scrolled_console_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
    metadata: &ConsoleOutputPaintMetadata,
    origin: &FrameRect,
    x: f32,
    y: f32,
    scroll_px: f32,
    popup_index: Option<&HostPaneTemplateHitIndex>,
) -> Option<TemplateNodePointerHit> {
    match hit_test_template_popup_rows(nodes, popup_index, origin, x, y) {
        Some(TemplatePopupRowHit::Hit(hit)) => return Some(hit),
        Some(TemplatePopupRowHit::Blocked) => return None,
        None => {}
    }

    let scroll_px = scroll_px.max(0.0);
    for row in metadata
        .visible_line_node_rows(nodes.row_count(), scroll_px)
        .rev()
    {
        let node = nodes.get(row)?;
        if !is_dispatchable(node) {
            continue;
        }
        let mut frame = template_node_frame(node, origin);
        frame.y -= scroll_px;
        if frame.width <= 0.0 || frame.height <= 0.0 || !contains_point(&frame, x, y) {
            continue;
        }
        return Some(template_node_pointer_hit_with_frame(node, frame, y));
    }
    None
}

pub(super) fn hit_test_console_static_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
    metadata: &ConsoleOutputPaintMetadata,
    origin: &FrameRect,
    x: f32,
    y: f32,
    popup_index: Option<&HostPaneTemplateHitIndex>,
) -> Option<TemplateNodePointerHit> {
    match hit_test_template_popup_rows(nodes, popup_index, origin, x, y) {
        Some(TemplatePopupRowHit::Hit(hit)) => return Some(hit),
        Some(TemplatePopupRowHit::Blocked) => return None,
        None => {}
    }

    let line_rows = metadata.line_node_rows(nodes.row_count());
    for row in (0..nodes.row_count()).rev() {
        if line_rows.contains(&row) {
            continue;
        }
        let node = nodes.get(row)?;
        if is_dispatchable(node) && template_node_accepts_point(node, origin, x, y) {
            return Some(template_node_pointer_hit(node, origin, y));
        }
    }
    None
}

pub(super) fn hit_test_workbench_template_nodes_with_index(
    nodes: &ModelRc<TemplatePaneNodeData>,
    index: &HostWorkbenchHitIndex,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    record_current_ui_perf_counter(UiPerfCounter::WorkbenchHitIndexQueryCount, 1.0);
    index.begin_query();
    let origin = index.origin()?;
    for row in index.popup_rows().iter().rev().copied() {
        let node = nodes.get(row)?;
        match hit_test_template_popup_node(node, origin, x, y) {
            Some(TemplatePopupRowHit::Hit(hit)) => return Some(hit),
            Some(TemplatePopupRowHit::Blocked) => return None,
            None => {}
        }
    }
    for row in index.candidate_rows(x, y).iter().rev().copied() {
        index.record_candidate_visit();
        let Some(node) = nodes.get(row) else {
            continue;
        };
        if is_dispatchable(node) && template_node_accepts_point(node, origin, x, y) {
            return Some(template_node_pointer_hit(node, origin, y));
        }
    }
    None
}

fn template_node_accepts_point(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> bool {
    let frame = template_node_frame(node, origin);
    if frame.width <= 0.0
        || frame.height <= 0.0
        || !contains_point(origin, x, y)
        || !contains_point(&frame, x, y)
    {
        return false;
    }
    if !node.has_clip_frame {
        return true;
    }
    let clip = FrameRect {
        x: origin.x + node.clip_frame.x,
        y: origin.y + node.clip_frame.y,
        width: node.clip_frame.width,
        height: node.clip_frame.height,
    };
    clip.width > 0.0 && clip.height > 0.0 && contains_point(&clip, x, y)
}

fn template_node_pointer_hit(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    y: f32,
) -> TemplateNodePointerHit {
    let frame = template_node_frame(node, origin);
    template_node_pointer_hit_with_frame(node, frame, y)
}

fn template_node_pointer_hit_with_frame(
    node: &TemplatePaneNodeData,
    frame: FrameRect,
    y: f32,
) -> TemplateNodePointerHit {
    let component_family = template_component_family(node);
    let table_row = (component_family == Some(TemplateComponentFamily::Table))
        .then(|| table_row_hit(node, &frame, y))
        .flatten();
    TemplateNodePointerHit {
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
    }
}

fn template_node_frame(node: &TemplatePaneNodeData, origin: &FrameRect) -> FrameRect {
    FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
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
