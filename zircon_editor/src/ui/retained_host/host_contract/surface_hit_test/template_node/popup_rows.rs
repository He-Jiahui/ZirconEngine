use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::frame_geometry::contains_point;
use super::super::super::paint_geometry::frame_from_template;
use super::super::super::template_component_family::template_component_family;
use super::super::super::template_popup_layout::{
    menu_item_row_frame, template_option_popup_frame_within, template_option_row_frame_within,
};
use super::TemplateNodePointerHit;

pub(in crate::ui::retained_host::host_contract) enum TemplatePopupRowHit {
    Hit(TemplateNodePointerHit),
    Blocked,
}

pub(in crate::ui::retained_host::host_contract) fn hit_test_template_popup_rows(
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    for row in (0..nodes.row_count()).rev() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !node.popup_open || node.disabled || node.control_id.is_empty() {
            continue;
        }
        if let Some(hit) = hit_test_template_menu_rows(&node, origin, x, y) {
            return Some(hit);
        }
        if let Some(hit) = hit_test_template_option_rows(&node, origin, x, y) {
            return Some(hit);
        }
    }
    None
}

fn hit_test_template_option_rows(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        return None;
    }
    let action_id = if node.edit_action_id.is_empty() {
        node.action_id.clone()
    } else {
        node.edit_action_id.clone()
    };
    if action_id.is_empty() {
        return None;
    }

    let local = frame_from_template(&node.frame);
    let control_frame = FrameRect {
        x: origin.x + local.x,
        y: origin.y + local.y,
        width: local.width,
        height: local.height,
    };
    let popup_frame = template_option_popup_frame_within(node, &control_frame, row_count, origin)?;
    for row in 0..row_count {
        let option = node.structured_options.row_data(row)?;
        if option.disabled {
            continue;
        }
        let row_frame =
            template_option_row_frame_within(node, &control_frame, row_count, row, origin)?;
        if contains_point(&row_frame, x, y) {
            return Some(TemplatePopupRowHit::Hit(template_popup_row_hit(
                node,
                row_frame,
                "workbench_option",
                action_id,
                option.id,
            )));
        }
    }
    if contains_point(&popup_frame, x, y) {
        return Some(TemplatePopupRowHit::Blocked);
    }
    None
}

fn hit_test_template_menu_rows(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    let row_count = node.structured_menu_items.row_count();
    if row_count == 0 {
        return None;
    }

    let local = frame_from_template(&node.frame);
    let menu_frame = FrameRect {
        x: origin.x + local.x,
        y: origin.y + local.y,
        width: local.width,
        height: local.height,
    };
    for row in 0..row_count {
        let item = node.structured_menu_items.row_data(row)?;
        if item.disabled || item.separator || item.action_id.is_empty() {
            continue;
        }
        let row_frame = menu_item_row_frame(&menu_frame, row_count, row)?;
        if contains_point(&row_frame, x, y) {
            return Some(TemplatePopupRowHit::Hit(template_popup_row_hit(
                node,
                row_frame,
                "workbench_menu_item",
                normalized_menu_row_action_id(item.action_id.as_str(), item.label.as_str()),
                item.label.clone(),
            )));
        }
    }
    if contains_point(&menu_frame, x, y) {
        return Some(TemplatePopupRowHit::Blocked);
    }
    None
}

fn template_popup_row_hit(
    node: &TemplatePaneNodeData,
    frame: FrameRect,
    dispatch_kind: &str,
    action_id: SharedString,
    value_text: SharedString,
) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        control_id: node.control_id.clone(),
        action_id,
        binding_id: String::new(),
        dispatch_kind: dispatch_kind.to_string(),
        component_role: node.component_role.clone(),
        component_family: template_component_family(node),
        value_text,
        edit_action_id: node.edit_action_id.clone(),
        commit_action_id: node.commit_action_id.clone(),
        frame,
    }
}

fn normalized_menu_row_action_id(action_id: &str, label: &str) -> SharedString {
    if action_id.starts_with("menu.item.") {
        return action_id.into();
    }
    menu_item_action_id(if label.is_empty() { action_id } else { label }).into()
}

fn menu_item_action_id(label: &str) -> String {
    format!("menu.item.{}", label_to_action_segment(label))
}

fn label_to_action_segment(label: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}
