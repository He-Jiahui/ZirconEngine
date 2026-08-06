mod action_id;
mod hit;
mod menu;
mod option;

use crate::ui::retained_host::primitives::ModelRc;

use self::menu::hit_test_template_menu_rows;
use self::option::hit_test_template_option_rows;
use super::super::super::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract) use self::hit::TemplatePopupRowHit;

pub(in crate::ui::retained_host::host_contract) fn hit_test_template_popup_rows(
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    for node in nodes.iter().rev() {
        if let Some(hit) = hit_test_template_popup_node(node, origin, x, y) {
            return Some(hit);
        }
    }
    None
}

pub(super) fn hit_test_template_popup_node(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    if !node.popup_open || node.disabled || node.control_id.is_empty() {
        return None;
    }
    hit_test_template_menu_rows(node, origin, x, y)
        .or_else(|| hit_test_template_option_rows(node, origin, x, y))
}

pub(super) fn uniform_popup_row_at_y(
    y: f32,
    top: f32,
    row_height: f32,
    row_count: usize,
) -> Option<usize> {
    if row_count == 0 || !y.is_finite() || !top.is_finite() || row_height <= 0.0 {
        return None;
    }
    let offset = y - top;
    if offset < 0.0 || offset > row_height * row_count as f32 {
        return None;
    }
    let row = if offset == 0.0 {
        0
    } else {
        ((offset / row_height).ceil() as usize).saturating_sub(1)
    };
    (row < row_count).then_some(row)
}

pub(super) fn next_uniform_popup_row_at_boundary(
    y: f32,
    top: f32,
    row_height: f32,
    row: usize,
    row_count: usize,
) -> Option<usize> {
    let next = row.checked_add(1)?;
    (next < row_count && y == top + row_height * next as f32).then_some(next)
}

#[cfg(test)]
mod performance_tests {
    use super::{next_uniform_popup_row_at_boundary, uniform_popup_row_at_y};

    #[test]
    fn uniform_popup_row_lookup_is_constant_time_and_preserves_inclusive_boundaries() {
        assert_eq!(uniform_popup_row_at_y(10.0, 10.0, 24.0, 10_000), Some(0));
        assert_eq!(uniform_popup_row_at_y(34.0, 10.0, 24.0, 10_000), Some(0));
        assert_eq!(
            next_uniform_popup_row_at_boundary(34.0, 10.0, 24.0, 0, 10_000),
            Some(1)
        );
        assert_eq!(uniform_popup_row_at_y(58.1, 10.0, 24.0, 10_000), Some(2));
        assert_eq!(uniform_popup_row_at_y(9.9, 10.0, 24.0, 10_000), None);
    }
}
