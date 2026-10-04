use super::*;

#[test]
fn tooltip_intro_progress_and_status_are_render_only() {
    for (property, value_kind) in [
        ("transition_progress", UiValueKind::Float),
        ("transition_status", UiValueKind::String),
    ] {
        assert_eq!(
            metadata_attribute_dirty("Tooltip", property, value_kind),
            render_dirty(),
            "tooltip intro samples must not rebuild layout, hit-test, text, or input"
        );
    }
}
