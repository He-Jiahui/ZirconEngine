use super::*;

fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

#[test]
fn pane_without_damage_clip_keeps_visible_content() {
    assert!(pane_intersects_damage(&rect(10.0, 10.0, 100.0, 80.0), None));
}

#[test]
fn pane_damage_gate_accepts_intersection_and_rejects_disjoint_or_collapsed_content() {
    let content = rect(10.0, 10.0, 100.0, 80.0);

    assert!(pane_intersects_damage(
        &content,
        Some(&rect(50.0, 20.0, 20.0, 20.0))
    ));
    assert!(!pane_intersects_damage(
        &content,
        Some(&rect(200.0, 200.0, 20.0, 20.0))
    ));
    assert!(!pane_intersects_damage(&rect(10.0, 10.0, 0.0, 80.0), None));
}
