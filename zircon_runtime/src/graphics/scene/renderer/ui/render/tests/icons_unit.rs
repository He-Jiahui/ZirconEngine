use super::*;

#[test]
fn built_in_info_icon_paints_multiple_analytic_shapes() {
    let mut vertices = Vec::new();
    assert!(push_builtin_icon(
        &mut vertices,
        "info",
        UiFrame::new(0.0, 0.0, 24.0, 24.0),
        [1.0; 4],
        UiFrame::new(0.0, 0.0, 100.0, 100.0),
    ));
    assert!(vertices.len() >= 18);
    assert!(vertices.iter().all(|vertex| {
        vertex.position.iter().all(|value| value.is_finite())
            && vertex.local_position.iter().all(|value| value.is_finite())
    }));
}

#[test]
fn built_in_package_icon_is_not_the_generic_square() {
    let mut vertices = Vec::new();
    assert!(push_builtin_icon(
        &mut vertices,
        "package",
        UiFrame::new(0.0, 0.0, 24.0, 24.0),
        [1.0; 4],
        UiFrame::new(0.0, 0.0, 100.0, 100.0),
    ));
    assert!(vertices.len() > 6);
}

#[test]
fn built_in_arrow_up_icon_paints_a_send_affordance() {
    let mut vertices = Vec::new();
    assert!(push_builtin_icon(
        &mut vertices,
        "arrow-up@s",
        UiFrame::new(0.0, 0.0, 24.0, 24.0),
        [1.0; 4],
        UiFrame::new(0.0, 0.0, 100.0, 100.0),
    ));
    assert!(vertices.len() >= 18);
}

#[test]
fn unknown_icons_remain_an_explicit_fallback() {
    let mut vertices = Vec::new();
    assert!(!push_builtin_icon(
        &mut vertices,
        "project-custom-icon",
        UiFrame::new(0.0, 0.0, 24.0, 24.0),
        [1.0; 4],
        UiFrame::new(0.0, 0.0, 100.0, 100.0),
    ));
    assert!(vertices.is_empty());
}
