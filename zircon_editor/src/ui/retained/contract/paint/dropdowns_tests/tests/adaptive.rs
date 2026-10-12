use super::super::push_dropdown_commands;
use super::support::dropdown_node;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn collapsed_or_fully_clipped_dropdowns_emit_no_commands() {
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
            x: 140.0,
            width: 40.0,
            ..clip
        },
        0,
        1.0,
    ));
    assert!(commands.is_empty());
}

#[test]
fn partially_clipped_dropdown_keeps_commands_and_defers_visibility_to_the_clip() {
    let node = dropdown_node(false);
    let rect = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 104.0,
        height: 32.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_dropdown_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(
        commands.iter().any(|command| command.text.is_some()),
        "a partially visible dropdown keeps its label command for renderer clipping"
    );
    assert!(commands
        .iter()
        .all(|command| command.clip_frame.as_ref() == Some(&clip)));
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
