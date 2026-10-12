use super::*;

#[test]
fn content_label_reclaims_only_the_shared_action_reserve() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 40.0,
    };
    let icon = FrameRect {
        x: 24.0,
        y: 8.0,
        width: 16.0,
        height: 16.0,
    };
    let default = tree_label_rect(&rect, &icon);
    let content = tree_label_rect_for_variant(&rect, &icon, true);
    let metrics = tree_metrics();
    assert_eq!(default.x, content.x);
    assert_eq!(default.y, content.y);
    assert_eq!(default.height, content.height);
    assert!(
        (content.width - default.width - metrics.tree_action_size * 2.0 - metrics.tree_action_gap)
            .abs()
            < 0.001
    );
    assert!((content.x + content.width - (rect.width - metrics.tree_right_inset)).abs() < 0.001);
}
