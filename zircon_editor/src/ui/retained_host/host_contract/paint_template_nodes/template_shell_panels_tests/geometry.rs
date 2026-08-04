use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_shell_panel_commands;
use super::super::separators::pixel_aligned_rect;
use super::support::panel_node;

#[test]
fn shell_panels_that_round_to_zero_do_not_emit_paint_commands() {
    for rect in [
        FrameRect {
            x: 8.0,
            y: 6.0,
            width: 0.4,
            height: 24.0,
        },
        FrameRect {
            x: 8.0,
            y: 6.0,
            width: 120.0,
            height: 0.4,
        },
    ] {
        let mut commands = Vec::new();
        let node = panel_node(
            "WorkbenchWindowTopToolbar",
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        );

        assert!(push_shell_panel_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            8,
            1.0,
        ));
        assert!(commands.is_empty());
    }
}

#[test]
fn fully_clipped_shell_panel_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 120.0,
        height: 40.0,
    };
    let clip = FrameRect {
        x: 160.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
    };
    let mut commands = Vec::new();
    let node = panel_node(
        "WorkbenchWindowTopToolbar",
        rect.x,
        rect.y,
        rect.width,
        rect.height,
    );

    assert!(push_shell_panel_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        8,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn partially_clipped_shell_panel_keeps_only_clipped_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 120.0,
        height: 40.0,
    };
    let clip = FrameRect {
        x: 16.0,
        y: 8.0,
        width: 60.0,
        height: 20.0,
    };
    let mut commands = Vec::new();
    let node = panel_node(
        "WorkbenchWindowTopToolbar",
        rect.x,
        rect.y,
        rect.width,
        rect.height,
    );

    assert!(push_shell_panel_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        8,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(commands.iter().all(|command| command
        .clip_frame
        .as_ref()
        .is_some_and(|clip_frame| frame_is_within(&clip, clip_frame))));
}

#[test]
fn shell_panel_pixel_alignment_stays_inside_fractional_parent_bounds() {
    let parent = FrameRect {
        x: 8.4,
        y: 6.6,
        width: 80.4,
        height: 40.4,
    };

    let aligned = pixel_aligned_rect(&parent);

    assert!(aligned.x >= parent.x);
    assert!(aligned.y >= parent.y);
    assert!(aligned.right() <= parent.right());
    assert!(aligned.bottom() <= parent.bottom());
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
