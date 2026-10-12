use super::*;
use crate::ui::retained_host::detail_pointer::console_scroll_layout;

#[test]
fn repeated_surface_size_is_a_no_op() {
    let mut surface = ScrollSurfaceHostState::new();
    let size = UiSize::new(320.0, 180.0);

    assert!(surface.set_size(size));
    assert!(!surface.set_size(size));
}

#[test]
fn follow_tail_tracks_growth_until_the_user_scrolls_away() {
    let mut surface = ScrollSurfaceHostState::new();
    let size = UiSize::new(320.0, 100.0);

    surface.sync_following_tail(console_scroll_layout(size, 180.0));
    assert_eq!(surface.scroll_offset(), 80.0);

    assert!(surface.handle_scroll(UiPoint::new(24.0, 40.0), -20.0));
    assert_eq!(surface.scroll_offset(), 60.0);

    surface.sync_following_tail(console_scroll_layout(size, 220.0));
    assert_eq!(surface.scroll_offset(), 60.0);

    assert!(surface.handle_scroll(UiPoint::new(24.0, 40.0), 4096.0));
    assert_eq!(surface.scroll_offset(), 120.0);

    surface.sync_following_tail(console_scroll_layout(size, 240.0));
    assert_eq!(surface.scroll_offset(), 140.0);
}

#[test]
fn sync_reads_back_the_bridge_clamp_after_content_shrinks() {
    let mut surface = ScrollSurfaceHostState::new();
    let size = UiSize::new(320.0, 100.0);
    surface.sync(console_scroll_layout(size, 180.0));
    assert!(surface.handle_scroll(UiPoint::new(24.0, 40.0), 4096.0));
    assert_eq!(surface.scroll_offset(), 80.0);

    surface.sync(console_scroll_layout(size, 40.0));
    assert_eq!(surface.scroll_offset(), 0.0);
}
