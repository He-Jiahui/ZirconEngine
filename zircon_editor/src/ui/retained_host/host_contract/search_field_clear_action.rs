use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

use super::current_host_metrics;

pub(crate) fn search_field_clear_action_frame(field: UiFrame) -> Option<UiFrame> {
    if !field.x.is_finite()
        || !field.y.is_finite()
        || !field.width.is_finite()
        || !field.height.is_finite()
        || field.width <= 0.0
        || field.height <= 0.0
    {
        return None;
    }

    let metrics = current_host_metrics();
    let size = (metrics.row_height - metrics.gap_l)
        .max(metrics.font_body)
        .min(field.height)
        .round();
    let right = field.x + field.width - metrics.input_pad[1];
    let frame = UiFrame::new(
        right - size,
        field.y + (field.height - size).max(0.0) * 0.5,
        size,
        size,
    );
    (frame.x >= field.x
        && frame.y >= field.y
        && frame.x + frame.width <= field.x + field.width
        && frame.y + frame.height <= field.y + field.height)
        .then_some(frame)
}

pub(crate) fn search_field_clear_action_hit_test(field: UiFrame, point: UiPoint) -> bool {
    let Some(action) = search_field_clear_action_frame(field) else {
        return false;
    };
    point.x >= action.x
        && point.y >= action.y
        && point.x <= action.x + action.width
        && point.y <= action.y + action.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_action_stays_inside_the_trailing_field_edge() {
        let field = UiFrame::new(12.0, 10.0, 184.0, 32.0);
        let action = search_field_clear_action_frame(field)
            .expect("a standard search field should fit its clear action");

        assert!(action.x >= field.x);
        assert!(action.y >= field.y);
        assert!(action.x + action.width <= field.x + field.width);
        assert!(action.y + action.height <= field.y + field.height);
        assert!(search_field_clear_action_hit_test(
            field,
            UiPoint::new(
                action.x + action.width * 0.5,
                action.y + action.height * 0.5,
            )
        ));
    }

    #[test]
    fn clear_action_rejects_points_outside_its_trailing_target() {
        let field = UiFrame::new(12.0, 10.0, 184.0, 32.0);

        assert!(!search_field_clear_action_hit_test(
            field,
            UiPoint::new(field.x + 4.0, field.y + field.height * 0.5)
        ));
    }

    #[test]
    fn clear_action_scales_down_for_a_compact_field() {
        let field = UiFrame::new(12.0, 10.0, 44.0, 18.0);
        let action = search_field_clear_action_frame(field)
            .expect("a compact search field should retain its clear action");

        assert!(action.height <= field.height);
        assert!(action.x >= field.x);
        assert!(action.y >= field.y);
        assert!(action.x + action.width <= field.x + field.width);
        assert!(action.y + action.height <= field.y + field.height);
        assert!(search_field_clear_action_hit_test(
            field,
            UiPoint::new(
                action.x + action.width * 0.5,
                action.y + action.height * 0.5,
            )
        ));
    }
}
