mod action_id;
mod hit;
mod menu;
mod option;
mod settings;

use crate::ui::retained_host::primitives::ModelRc;

use self::menu::hit_test_template_menu_row_target;
use self::option::hit_test_template_option_row_target;
use self::settings::hit_test_settings_window_target;
use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::HostPaneTemplateHitIndex;

pub(super) use self::action_id::normalized_menu_row_action_id;
pub(super) use self::hit::TemplatePopupRowTarget;
pub(in crate::ui::retained_host::host_contract) use self::hit::{
    TemplatePopupRowHit, TemplatePopupRowMoveHit, TemplatePopupRowRouteHit,
};

pub(in crate::ui::retained_host::host_contract) fn hit_test_template_popup_route_rows<'a>(
    nodes: &'a ModelRc<TemplatePaneNodeData>,
    index: Option<&HostPaneTemplateHitIndex>,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowRouteHit<'a>> {
    if let Some(index) = index {
        index.begin_query();
        for row in index.popup_rows().iter().rev().copied() {
            index.record_popup_candidate_visit();
            let node = nodes.get(row)?;
            if let Some(target) = hit_test_template_popup_target(node, origin, x, y) {
                return Some(target.into_pointer_route_hit(node));
            }
        }
        return None;
    }
    for node in nodes.iter().rev() {
        if let Some(target) = hit_test_template_popup_target(node, origin, x, y) {
            return Some(target.into_pointer_route_hit(node));
        }
    }
    None
}

pub(super) fn hit_test_template_popup_target<'a>(
    node: &'a TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowTarget<'a>> {
    if !node.popup_open || node.disabled || node.control_id.is_empty() {
        return None;
    }
    hit_test_settings_window_target(node, origin, x, y)
        .or_else(|| hit_test_template_menu_row_target(node, origin, x, y))
        .or_else(|| hit_test_template_option_row_target(node, origin, x, y))
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
