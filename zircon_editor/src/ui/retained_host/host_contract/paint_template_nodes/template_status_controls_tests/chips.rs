use super::super::super::super::data::FrameRect;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::{
    PALETTE, push_status_control_commands, select_workbench_status_chip_style,
    status_chip_text_colors, status_chip_text_rect, status_control_offset_rect,
};
use super::support::{pixel_at, status_chip_node};
use crate::ui::layouts::common::model_rc;

const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

#[test]
fn status_chip_paints_flat_text_without_surface_or_chevron() {
    let bytes = paint_template_nodes_for_test(
        140,
        48,
        model_rc(vec![status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm")]),
    );

    assert_eq!(pixel_at(&bytes, 140, 12, 12), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 140, 60, 9), [0, 0, 0, 255]);
}

#[test]
fn status_chip_emits_only_text_commands_for_normal_state() {
    let node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    let rect = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    let mut commands = Vec::new();

    push_status_control_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert_eq!(
        commands
            .iter()
            .filter(|command| command.text.is_some())
            .count(),
        2
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.text.is_none())
            .count(),
        0
    );
}

#[test]
fn status_chip_uses_declared_text_colors_and_layout_offset() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.layout_offset_y = -2.0;
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(91, 101, 107);
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144);

    let rect = status_control_offset_rect(
        &node,
        &FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
    );

    assert!((rect.y - 7.0).abs() < 0.001);
    assert_eq!(
        status_chip_text_colors(&node),
        ([91, 101, 107, 255], [125, 137, 144, 255])
    );
}

#[test]
fn status_chip_right_aligns_colon_value_text() {
    let node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    let rect = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: 144.0,
        height: node.frame.height,
    };
    let base_text = status_chip_text_rect(&rect);
    let mut commands = Vec::new();

    push_status_control_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let text_commands = commands
        .iter()
        .filter(|command| command.text.is_some())
        .collect::<Vec<_>>();
    assert_eq!(text_commands.len(), 2);
    assert_eq!(text_commands[0].text.as_deref(), Some("Grid:"));
    assert_eq!(text_commands[1].text.as_deref(), Some("10 cm"));
    assert!(
        ((text_commands[1].frame.x + text_commands[1].frame.width)
            - (base_text.x + base_text.width))
            .abs()
            < 0.01
    );
    assert!(text_commands[0].frame.x < text_commands[1].frame.x);
}

#[test]
fn status_bar_right_value_controls_use_status_chip_text_layers() {
    for (control_id, text, expected_count) in [
        ("WorkbenchStatusGrid", "Grid: 10 cm", 2),
        ("WorkbenchStatusSnap", "Snap: On", 2),
        ("WorkbenchStatusZoom", "100%", 1),
    ] {
        let node = status_chip_node(control_id, text);
        let rect = FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let mut commands = Vec::new();

        push_status_control_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

        let text_commands = commands
            .iter()
            .filter(|command| command.text.is_some())
            .collect::<Vec<_>>();
        assert_eq!(text_commands.len(), expected_count, "{control_id}");
        assert_eq!(
            commands
                .iter()
                .filter(|command| command.text.is_none())
                .count(),
            0,
            "{control_id}"
        );
        if expected_count == 2 {
            assert_eq!(
                text_commands[0].foreground_color,
                Some(PALETTE.text_muted),
                "{control_id}"
            );
            assert_eq!(
                text_commands[1].foreground_color,
                Some(PALETTE.text),
                "{control_id}"
            );
        } else {
            assert_eq!(
                text_commands[0].foreground_color,
                Some(PALETTE.text),
                "{control_id}"
            );
        }
    }
}

#[test]
fn status_chip_right_aligns_value_only_text() {
    let node = status_chip_node("WorkbenchStatusZoom", "100%");
    let rect = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: 88.0,
        height: node.frame.height,
    };
    let base_text = status_chip_text_rect(&rect);
    let mut commands = Vec::new();

    push_status_control_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let text_commands = commands
        .iter()
        .filter(|command| command.text.is_some())
        .collect::<Vec<_>>();
    assert_eq!(text_commands.len(), 1);
    assert_eq!(text_commands[0].text.as_deref(), Some("100%"));
    assert!(
        ((text_commands[0].frame.x + text_commands[0].frame.width)
            - (base_text.x + base_text.width))
            .abs()
            < 0.01
    );
    assert!(text_commands[0].frame.x > base_text.x);
}

#[test]
fn status_chip_uses_shared_painter_state_priority() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.hovered = true;
    node.selected = true;
    let hovered = select_workbench_status_chip_style(&node);
    assert_eq!(
        hovered.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Selected
    );
    assert_eq!(hovered.background, PALETTE.surface_selected);

    node.pressed = true;
    let pressed = select_workbench_status_chip_style(&node);
    assert_eq!(
        pressed.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(pressed.border, PALETTE.focus_ring);

    node.disabled = true;
    let disabled = select_workbench_status_chip_style(&node);
    assert_eq!(
        disabled.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Disabled
    );
    assert_eq!(disabled.background, PALETTE.surface_disabled);
}

#[test]
fn status_chip_normal_state_is_flat_transparent_status_text() {
    let node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");

    let normal = select_workbench_status_chip_style(&node);

    assert_eq!(normal.background, TRANSPARENT);
    assert_eq!(normal.border, TRANSPARENT);
    assert_eq!(normal.label_text, PALETTE.text_muted);
    assert_eq!(normal.value_text, PALETTE.text);
}

#[test]
fn focused_status_chip_keeps_flat_surface_with_focus_border() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.focused = true;

    let focused = select_workbench_status_chip_style(&node);

    assert_eq!(
        focused.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Focused
    );
    assert_eq!(focused.background, TRANSPARENT);
    assert_eq!(focused.border, PALETTE.focus_ring);
    assert_eq!(focused.label_text, PALETTE.text_muted);
    assert_eq!(focused.value_text, PALETTE.text);
}

#[test]
fn focused_hovered_status_chip_uses_hover_surface() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.focused = true;
    node.hovered = true;

    let focused = select_workbench_status_chip_style(&node);

    assert_eq!(
        focused.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Focused
    );
    assert_eq!(focused.background, PALETTE.surface_hover);
    assert_eq!(focused.border, PALETTE.focus_ring);
}

#[test]
fn focused_selected_status_chip_keeps_selected_surface() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.focused = true;
    node.selected = true;

    let focused = select_workbench_status_chip_style(&node);

    assert_eq!(
        focused.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Focused
    );
    assert_eq!(focused.background, PALETTE.surface_selected);
    assert_eq!(focused.border, PALETTE.focus_ring);
}
