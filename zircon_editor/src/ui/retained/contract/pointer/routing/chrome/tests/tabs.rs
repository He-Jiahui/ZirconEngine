use super::*;

#[test]
fn dock_overflow_route_uses_the_published_local_anchor_once() {
    let route = route_dock_overflow(
        "document",
        &FrameRect {
            x: 100.0,
            y: 40.0,
            ..FrameRect::default()
        },
        &FrameRect {
            x: 200.0,
            y: 2.0,
            width: 28.0,
            height: 28.0,
        },
        314.0,
        54.0,
    );

    assert!(matches!(
        route,
        Some(ChromePointerRoute::DockOverflow { surface_key })
            if surface_key == "document"
    ));
}
