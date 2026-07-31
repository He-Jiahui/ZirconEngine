use super::super::push_dropdown_commands;
use super::support::dropdown_node;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn collapsed_or_clip_escaping_dropdowns_emit_no_commands() {
    let node = dropdown_node(false);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_dropdown_commands(
        &mut commands,
        &node,
        &FrameRect {
            x: 12.0,
            y: 8.0,
            width: 0.0,
            height: 32.0,
        },
        &clip,
        0,
        1.0,
    ));
    assert!(commands.is_empty());

    assert!(push_dropdown_commands(
        &mut commands,
        &node,
        &FrameRect {
            x: 12.0,
            y: 8.0,
            width: 104.0,
            height: 32.0,
        },
        &FrameRect {
            width: 40.0,
            ..clip
        },
        0,
        1.0,
    ));
    assert!(commands.is_empty());
}

#[test]
fn narrow_or_short_dropdowns_skip_text_and_chevron_but_keep_the_surface() {
    let node = dropdown_node(false);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 80.0,
    };

    for rect in [
        FrameRect {
            x: 12.0,
            y: 8.0,
            width: 2.0,
            height: 32.0,
        },
        FrameRect {
            x: 12.0,
            y: 8.0,
            width: 104.0,
            height: 2.0,
        },
    ] {
        let mut commands = Vec::new();
        assert!(push_dropdown_commands(
            &mut commands,
            &node,
            &rect,
            &clip,
            0,
            1.0,
        ));
        assert_eq!(commands.len(), 1);
        assert!(commands[0].text.is_none());
        assert!(commands[0].image_pixels.is_none());
    }
}
