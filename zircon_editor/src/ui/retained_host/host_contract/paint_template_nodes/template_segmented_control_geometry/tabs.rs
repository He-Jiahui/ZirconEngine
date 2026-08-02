use super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_geometry::bounded_extent,
};
use super::metrics::{tab_line_height, tab_text_inset_x, tab_underline_height};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: bounded_extent(rect.width),
        height: bounded_extent(rect.height),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_underline_rect(
    rect: &FrameRect,
) -> FrameRect {
    let available_height = bounded_extent(rect.height);
    let underline_height = bounded_extent(tab_underline_height()).min(available_height);
    FrameRect {
        x: rect.x,
        y: rect.y + (available_height - underline_height).max(0.0),
        width: bounded_extent(rect.width),
        height: underline_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    let inset_x = tab_text_inset_x();
    let line_height = tab_line_height();
    let available_height = bounded_extent(rect.height);
    let height = bounded_extent(line_height).min(available_height);
    FrameRect {
        x: rect.x + inset_x,
        y: rect.y + (available_height - height) * 0.5,
        width: bounded_extent(rect.width - inset_x * 2.0),
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_tab_has_no_paint_label_or_underline_extent() {
        let tab = FrameRect {
            x: 12.0,
            y: 8.0,
            width: 0.0,
            height: 0.0,
        };

        let painted = tab_paint_rect(&TemplatePaneNodeData::default(), &tab);
        let label = tab_label_rect(&tab);
        let underline = tab_underline_rect(&tab);

        assert_eq!((painted.width, painted.height), (0.0, 0.0));
        assert_eq!((label.width, label.height), (0.0, 0.0));
        assert_eq!((underline.width, underline.height), (0.0, 0.0));
    }

    #[test]
    fn non_finite_tab_extent_does_not_produce_a_non_finite_underline_origin() {
        let underline = tab_underline_rect(&FrameRect {
            x: 12.0,
            y: 8.0,
            width: f32::NAN,
            height: f32::INFINITY,
        });

        assert_eq!(underline.x, 12.0);
        assert_eq!(underline.y, 8.0);
        assert_eq!((underline.width, underline.height), (0.0, 0.0));
    }
}
