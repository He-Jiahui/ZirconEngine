use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};
use super::super::metrics::segment_selected_inset;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_rect(
    segment: &FrameRect,
) -> FrameRect {
    inset_rect(segment, segment_selected_inset())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_underline_rect(
    selected_rect: &FrameRect,
    underline_height: f32,
) -> FrameRect {
    FrameRect {
        x: selected_rect.x,
        y: selected_rect.y + (selected_rect.height - underline_height).max(0.0),
        width: bounded_extent(selected_rect.width),
        height: bounded_extent(underline_height).min(bounded_extent(selected_rect.height)),
    }
}

fn inset_rect(rect: &FrameRect, inset: f32) -> FrameRect {
    let inset = bounded_extent(inset);
    FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: bounded_extent(rect.width - inset * 2.0),
        height: bounded_extent(rect.height - inset * 2.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_selected_segment_has_no_indicator_extent() {
        let segment = FrameRect {
            x: 12.0,
            y: 8.0,
            width: 0.0,
            height: 0.0,
        };

        let selected = selected_segment_rect(&segment);
        let underline = selected_segment_underline_rect(&selected, 3.0);

        assert_eq!((selected.width, selected.height), (0.0, 0.0));
        assert_eq!((underline.width, underline.height), (0.0, 0.0));
    }
}
