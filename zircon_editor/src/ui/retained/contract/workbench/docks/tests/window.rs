use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

#[test]
fn floating_window_palette_projects_all_window_roles_from_the_current_theme() {
    let mut palette = PALETTE;
    palette.shadow = [1, 2, 3, 255];
    palette.surface = [4, 5, 6, 255];
    palette.focus_ring = [7, 8, 9, 255];
    palette.popup = [10, 11, 12, 255];

    assert_eq!(
        floating_window_palette(palette),
        FloatingWindowPalette {
            shadow: [1, 2, 3, 255],
            panel: [4, 5, 6, 255],
            border: [7, 8, 9, 255],
            header: [10, 11, 12, 255],
        }
    );
}

#[test]
fn floating_window_shadow_uses_the_shared_slate_gap_and_border_metrics() {
    let shadow = floating_window_shadow_frame(
        &FrameRect {
            x: 20.0,
            y: 30.0,
            width: 200.0,
            height: 120.0,
        },
        METRICS,
    );

    assert_eq!(shadow.x, 24.0);
    assert_eq!(shadow.y, 35.0);
    assert_eq!(shadow.width, 200.0);
    assert_eq!(shadow.height, 120.0);
}

#[test]
fn floating_window_paint_bounds_include_the_shadow_only_region() {
    let window = FrameRect {
        x: 20.0,
        y: 30.0,
        width: 200.0,
        height: 120.0,
    };

    let bounds = floating_window_paint_bounds(&window);

    assert_eq!(bounds.x, window.x);
    assert_eq!(bounds.y, window.y);
    assert!(bounds.right() > window.right());
    assert!(bounds.bottom() > window.bottom());
}

#[test]
fn floating_window_body_frame_stays_inside_the_border_and_below_the_header() {
    let body = floating_window_body_frame(
        &FrameRect {
            x: 20.0,
            y: 30.0,
            width: 200.0,
            height: 120.0,
        },
        &FrameRect {
            x: 20.0,
            y: 30.0,
            width: 200.0,
            height: 28.0,
        },
        1.0,
    );

    assert_eq!(body.x, 21.0);
    assert_eq!(body.y, 59.0);
    assert_eq!(body.width, 198.0);
    assert_eq!(body.height, 90.0);
}
