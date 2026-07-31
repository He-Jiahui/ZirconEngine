use super::super::super::super::data::FrameRect;
use super::super::push_template_popup_row_commands;
use super::support::popup_menu_node;

#[test]
fn collapsed_or_clip_escaping_popup_row_roots_emit_no_commands() {
    let node = popup_menu_node();
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 180.0,
    };

    let mut commands = Vec::new();
    push_template_popup_row_commands(
        &mut commands,
        &node,
        &FrameRect {
            x: 10.0,
            y: 10.0,
            width: 0.0,
            height: 80.0,
        },
        &bounds,
        &bounds,
        0,
        1.0,
    );
    assert!(commands.is_empty());

    push_template_popup_row_commands(
        &mut commands,
        &node,
        &FrameRect {
            x: 10.0,
            y: 10.0,
            width: 120.0,
            height: 80.0,
        },
        &bounds,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 40.0,
            height: 80.0,
        },
        0,
        1.0,
    );
    assert!(commands.is_empty());
}

#[test]
fn short_popup_rows_keep_runtime_text_commands_out_of_collapsed_lines() {
    let node = popup_menu_node();
    let rect = FrameRect {
        x: 10.0,
        y: 10.0,
        width: 130.0,
        height: 10.0,
    };
    let mut commands = Vec::new();

    push_template_popup_row_commands(&mut commands, &node, &rect, &rect, &rect, 0, 1.0);

    assert!(
        !commands.is_empty(),
        "the valid popup root keeps its surface"
    );
    assert!(commands.iter().all(|command| command.text.is_none()));
}
