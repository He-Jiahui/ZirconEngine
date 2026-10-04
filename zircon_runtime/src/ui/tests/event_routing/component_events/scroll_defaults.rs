//! 滚轮默认动作保持原路由、偏移和祖先回退合同。

use super::*;

#[test]
fn scroll_fallback_reports_scroll_defaulted_when_unhandled() {
    let mut surface = scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.scroll_defaulted);
}

#[test]
fn routed_pointer_input_preserves_scroll_delta_and_reuses_the_default_scroll_authority() {
    let mut surface = scrollable_surface();
    let route = surface
        .route_pointer_input_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(route.scroll_delta, 50.0);
    assert_eq!(
        surface.apply_default_pointer_scroll(&route).unwrap(),
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        50.0
    );
}

#[test]
fn scroll_fallback_does_not_handle_when_scroll_offset_is_unchanged() {
    let mut surface = scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(0.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, None);
    assert!(!result.diagnostics.scroll_defaulted);
}

#[test]
fn scroll_fallback_continues_to_ancestor_when_nearest_scrollable_is_clamped() {
    let mut surface = nested_scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(20.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.scroll_defaulted);
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        20.0
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        0.0
    );
}
