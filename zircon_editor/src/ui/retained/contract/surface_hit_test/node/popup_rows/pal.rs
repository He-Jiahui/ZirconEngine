use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::frame_geometry::contains_point;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::template_popup_layout::{
    command_palette_metrics, command_palette_option_frame, command_palette_option_frame_within,
    command_palette_search_frame,
};
use super::super::TemplateNodePointerMoveKind;
use super::hit::TemplatePopupRowTarget;
use super::{next_uniform_popup_row_at_boundary, uniform_popup_row_at_y};

pub(super) fn hit_test_command_palette_target<'a>(
    node: &'a TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowTarget<'a>> {
    let panel = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    let metrics = command_palette_metrics();
    if panel.width <= metrics.min_frame_extent
        || panel.height <= metrics.min_frame_extent
        || !contains_point(&panel, x, y)
    {
        return None;
    }
    let mut clip = intersect(&panel, origin)?;
    if node.has_clip_frame {
        clip = intersect(
            &clip,
            &FrameRect {
                x: origin.x + node.clip_frame.x,
                y: origin.y + node.clip_frame.y,
                width: node.clip_frame.width,
                height: node.clip_frame.height,
            },
        )?;
    }
    if !contains_point(&clip, x, y) {
        return None;
    }
    let search = intersect(&command_palette_search_frame(&panel), &clip);
    if let Some(search) = search.filter(|frame| contains_point(frame, x, y)) {
        return Some(if node.edit_action_id.is_empty() {
            TemplatePopupRowTarget::Blocked
        } else {
            TemplatePopupRowTarget::TextInput {
                control_id: node.control_id.as_str(),
                dispatch_kind: "surface",
                edit_action_id: node.edit_action_id.as_str(),
                commit_action_id: node.commit_action_id.as_str(),
                value_text: node.search_query.as_str(),
                frame: search,
            }
        });
    }
    let row_count = node.structured_options.row_count();
    let first = command_palette_option_frame(&panel, 0);
    let Some(row) = uniform_popup_row_at_y(y, first.y, first.height, row_count) else {
        return Some(TemplatePopupRowTarget::Blocked);
    };
    if node.commit_action_id.is_empty() {
        return Some(TemplatePopupRowTarget::Blocked);
    }
    for candidate in [
        Some(row),
        next_uniform_popup_row_at_boundary(y, first.y, first.height, row, row_count),
    ]
    .into_iter()
    .flatten()
    {
        let option = node.structured_options.get(candidate)?;
        if option.disabled {
            continue;
        }
        let Some(frame) = command_palette_option_frame_within(&panel, candidate, &clip)
            .filter(|frame| contains_point(frame, x, y))
        else {
            continue;
        };
        return Some(TemplatePopupRowTarget::Hit {
            kind: TemplateNodePointerMoveKind::Option,
            action_id: node.commit_action_id.as_str(),
            value_text: option.id.as_str(),
            frame,
        });
    }
    Some(TemplatePopupRowTarget::Blocked)
}
