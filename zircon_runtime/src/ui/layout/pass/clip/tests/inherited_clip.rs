use super::resolve_clip_frame;
use zircon_runtime_interface::ui::layout::UiFrame;

#[test]
fn disjoint_clip_remains_empty_through_clipping_and_nonclipping_descendants() {
    let ancestor = UiFrame::new(0.0, 0.0, 10.0, 10.0);
    let outside = UiFrame::new(20.0, 20.0, 5.0, 5.0);
    let empty = resolve_clip_frame(Some(ancestor), outside, true)
        .expect("an empty inherited clip must remain distinct from no clip");
    assert!(empty.width <= 0.0 && empty.height <= 0.0);
    assert_eq!(
        resolve_clip_frame(Some(empty), ancestor, false),
        Some(empty)
    );
    assert_eq!(
        resolve_clip_frame(Some(empty), ancestor, true),
        Some(UiFrame::default())
    );
}

#[test]
fn nonclipping_nodes_preserve_ancestor_clip_for_overflowing_children() {
    let ancestor = UiFrame::new(0.0, 0.0, 100.0, 100.0);
    let child = UiFrame::new(20.0, 20.0, 10.0, 10.0);
    assert_eq!(
        resolve_clip_frame(Some(ancestor), child, false),
        Some(ancestor)
    );
    let outside_parent = UiFrame::new(60.0, 60.0, 10.0, 10.0);
    assert_eq!(
        resolve_clip_frame(Some(ancestor), outside_parent, true),
        Some(outside_parent)
    );
}

#[test]
fn clipping_nodes_use_the_positive_intersection_and_no_clip_is_preserved() {
    let ancestor = UiFrame::new(0.0, 0.0, 10.0, 10.0);
    let partial = UiFrame::new(8.0, 7.0, 8.0, 9.0);
    assert_eq!(
        resolve_clip_frame(Some(ancestor), partial, true),
        Some(UiFrame::new(8.0, 7.0, 2.0, 3.0))
    );
    assert_eq!(resolve_clip_frame(None, partial, true), Some(partial));
    assert_eq!(resolve_clip_frame(None, partial, false), None);
}
