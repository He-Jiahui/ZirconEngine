use super::data::{FrameRect, TemplatePaneNodeData};

const TEMPLATE_POPUP_ROW_GAP: f32 = 4.0;
const MIN_TEMPLATE_POPUP_ROW_HEIGHT: f32 = 24.0;

pub(crate) fn dropdown_option_popup_frame(
    control_frame: &FrameRect,
    row_count: usize,
) -> Option<FrameRect> {
    if row_count == 0 {
        return None;
    }
    let row_height = dropdown_option_row_height(control_frame);
    Some(FrameRect {
        x: control_frame.x,
        y: control_frame.y + control_frame.height + TEMPLATE_POPUP_ROW_GAP,
        width: control_frame.width.max(1.0),
        height: row_height * row_count as f32,
    })
}

pub(crate) fn dropdown_option_popup_frame_within(
    control_frame: &FrameRect,
    row_count: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    let mut popup = dropdown_option_popup_frame(control_frame, row_count)?;
    if !valid_bounds(bounds) {
        return Some(popup);
    }

    let below_y = control_frame.y + control_frame.height + TEMPLATE_POPUP_ROW_GAP;
    let above_y = control_frame.y - TEMPLATE_POPUP_ROW_GAP - popup.height;
    let bounds_bottom = bounds.y + bounds.height;
    if below_y + popup.height > bounds_bottom && above_y >= bounds.y {
        popup.y = above_y;
    }

    let popup_width = popup.width.min(bounds.width.max(1.0)).max(1.0);
    let max_x = (bounds.x + bounds.width - popup_width).max(bounds.x);
    popup.x = popup.x.clamp(bounds.x, max_x);
    popup.width = popup_width;
    Some(popup)
}

pub(crate) fn template_option_popup_frame_within(
    node: &TemplatePaneNodeData,
    control_frame: &FrameRect,
    row_count: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if template_option_rows_use_projected_frame(node) {
        return (row_count > 0).then_some(control_frame.clone());
    }
    dropdown_option_popup_frame_within(control_frame, row_count, bounds)
}

pub(crate) fn dropdown_option_row_frame(control_frame: &FrameRect, row: usize) -> FrameRect {
    let row_height = dropdown_option_row_height(control_frame);
    FrameRect {
        x: control_frame.x,
        y: control_frame.y
            + control_frame.height
            + TEMPLATE_POPUP_ROW_GAP
            + row as f32 * row_height,
        width: control_frame.width.max(1.0),
        height: row_height,
    }
}

pub(crate) fn template_option_row_frame_within(
    node: &TemplatePaneNodeData,
    control_frame: &FrameRect,
    row_count: usize,
    row: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if row >= row_count {
        return None;
    }
    let popup = template_option_popup_frame_within(node, control_frame, row_count, bounds)?;
    let row_height = if template_option_rows_use_projected_frame(node) {
        menu_item_row_height(&popup, row_count)?
    } else {
        dropdown_option_row_height(control_frame)
    };
    Some(FrameRect {
        x: popup.x,
        y: popup.y + row as f32 * row_height,
        width: popup.width,
        height: row_height,
    })
}

pub(crate) fn dropdown_option_row_frame_within(
    control_frame: &FrameRect,
    row_count: usize,
    row: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if row >= row_count {
        return None;
    }
    let popup = dropdown_option_popup_frame_within(control_frame, row_count, bounds)?;
    let row_height = dropdown_option_row_height(control_frame);
    Some(FrameRect {
        x: popup.x,
        y: popup.y + row as f32 * row_height,
        width: popup.width,
        height: row_height,
    })
}

pub(crate) fn menu_item_row_frame(
    menu_frame: &FrameRect,
    row_count: usize,
    row: usize,
) -> Option<FrameRect> {
    let row_height = menu_item_row_height(menu_frame, row_count)?;
    Some(FrameRect {
        x: menu_frame.x,
        y: menu_frame.y + row as f32 * row_height,
        width: menu_frame.width.max(1.0),
        height: row_height,
    })
}

fn dropdown_option_row_height(control_frame: &FrameRect) -> f32 {
    control_frame.height.max(MIN_TEMPLATE_POPUP_ROW_HEIGHT)
}

fn menu_item_row_height(menu_frame: &FrameRect, row_count: usize) -> Option<f32> {
    (row_count > 0)
        .then_some((menu_frame.height / row_count as f32).max(MIN_TEMPLATE_POPUP_ROW_HEIGHT))
}

pub(crate) fn template_option_rows_use_projected_frame(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "DropdownPopup")
        || matches!(node.component_role.as_str(), "dropdown-popup")
}

fn valid_bounds(bounds: &FrameRect) -> bool {
    bounds.x.is_finite()
        && bounds.y.is_finite()
        && bounds.width.is_finite()
        && bounds.height.is_finite()
        && bounds.width > 0.0
        && bounds.height > 0.0
}

#[cfg(test)]
mod tests {
    use super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
    use super::*;

    #[test]
    fn dropdown_option_popup_frame_within_opens_above_when_below_overflows() {
        let control = rect(20.0, 120.0, 100.0, 28.0);
        let bounds = rect(0.0, 0.0, 160.0, 160.0);

        let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
            .expect("popup should have a frame");

        assert_eq!(popup.x, 20.0);
        assert_eq!(popup.y, 32.0);
        assert_eq!(popup.width, 100.0);
        assert_eq!(popup.height, 84.0);
    }

    #[test]
    fn dropdown_option_popup_frame_within_keeps_default_when_above_also_overflows() {
        let control = rect(20.0, 12.0, 100.0, 28.0);
        let bounds = rect(0.0, 0.0, 160.0, 72.0);

        let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
            .expect("popup should have a frame");

        assert_eq!(popup.y, 44.0);
    }

    #[test]
    fn dropdown_option_popup_frame_within_clamps_right_edge() {
        let control = rect(120.0, 20.0, 80.0, 28.0);
        let bounds = rect(0.0, 0.0, 160.0, 160.0);

        let popup = dropdown_option_popup_frame_within(&control, 2, &bounds)
            .expect("popup should have a frame");

        assert_eq!(popup.x, 80.0);
        assert_eq!(popup.width, 80.0);
    }

    #[test]
    fn template_option_popup_frame_within_uses_projected_dropdown_popup_frame() {
        let node = TemplatePaneNodeData {
            role: "DropdownPopup".into(),
            component_role: "dropdown-popup".into(),
            frame: TemplateNodeFrameData {
                x: 100.0,
                y: 60.0,
                width: 120.0,
                height: 96.0,
            },
            ..TemplatePaneNodeData::default()
        };
        let popup = template_option_popup_frame_within(
            &node,
            &rect(100.0, 60.0, 120.0, 96.0),
            4,
            &rect(0.0, 0.0, 320.0, 240.0),
        )
        .expect("DropdownPopup should use its projected popup frame");
        let row =
            template_option_row_frame_within(&node, &popup, 4, 2, &rect(0.0, 0.0, 320.0, 240.0))
                .expect("DropdownPopup row should be inside the projected popup frame");

        assert_eq!(popup, rect(100.0, 60.0, 120.0, 96.0));
        assert_eq!(row, rect(100.0, 108.0, 120.0, 24.0));
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
