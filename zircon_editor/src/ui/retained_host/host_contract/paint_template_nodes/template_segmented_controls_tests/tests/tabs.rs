use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_segmented_control_geometry::tab_paint_rect;
use super::super::identity::is_workbench_tab;
use super::super::push_segmented_control_commands;
use super::super::style::tab_background;
use super::support::{changed_pixel_count, frame_rect, pixel_at, tab_node};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn selected_tab_paints_accent_underline_over_panel_surface() {
    let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![tab_node()]));

    assert!(changed_pixel_count(&bytes, 180, 0, 40, 150, 4) > 0);
    assert_eq!(
        pixel_at(&bytes, 180, 148, 8),
        crate::ui::retained_host::host_contract::paint_theme::PALETTE.surface_panel
    );
}

#[test]
fn selected_tab_honors_declared_layout_offset() {
    let mut node = tab_node();
    node.control_id = "WorkbenchLabsTabOne".into();
    node.layout_offset_x = 3.0;
    node.layout_offset_y = 2.0;
    let paint_rect = tab_paint_rect(&node, &frame_rect(&node.frame));

    assert!(is_workbench_tab(&node));
    assert_eq!(paint_rect.x, 3.0);
    assert_eq!(paint_rect.y, 6.0);

    let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 180, 0, 44), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 180, 3, 44, 150, 2) > 0);
}

#[test]
fn workbench_tab_uses_declared_idle_background() {
    use zircon_runtime_interface::ui::style::{
        ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
    };

    let mut node = tab_node();
    node.control_id = "WorkbenchLabsTabs".into();
    node.text = "".into();
    node.checked = false;
    node.selected = false;
    node.button_style = ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(28, 34, 38, 255))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    };

    assert!(is_workbench_tab(&node));
    assert_eq!(tab_background(&node), Some([28, 34, 38, 255]));

    let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 180, 8, 12), [28, 34, 38, 255]);
}

#[test]
fn fully_clipped_workbench_tab_does_not_emit_paint_commands() {
    let node = tab_node();
    let rect = frame_rect(&node.frame);
    let clip = FrameRect {
        x: 160.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_segmented_control_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn partially_clipped_workbench_tab_keeps_runtime_text_commands() {
    let node = tab_node();
    let rect = frame_rect(&node.frame);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 24.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_segmented_control_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        4,
        1.0,
    ));

    assert!(commands.iter().any(|command| command.text.is_some()));
    assert!(commands
        .iter()
        .all(|command| command.clip_frame.as_ref() == Some(&clip)));
}

#[test]
fn visible_tab_focus_paints_one_pixel_inside_bounds_and_keeps_underline() {
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    let mut node = tab_node();
    node.text = "".into();
    node.focused = true;
    node.focus_visible_known = true;
    node.focus_visible = true;
    node.layout_offset_x = 3.0;
    node.layout_offset_y = 2.0;
    let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node.clone()]));
    // Full original bounds (3, 6, 150, 40), inner 1px stroke.
    for (x, y) in [(3, 12), (152, 12), (30, 6)] {
        assert_eq!(pixel_at(&bytes, 180, x, y), PALETTE.focus_ring);
    }
    for (x, y) in [(4, 12), (151, 12), (30, 7)] {
        assert_eq!(pixel_at(&bytes, 180, x, y), PALETTE.surface_panel);
    }
    for (x, y) in [(2, 12), (153, 12), (30, 5), (30, 46)] {
        assert_eq!(pixel_at(&bytes, 180, x, y), [0, 0, 0, 255]);
    }
    let mut commands = Vec::new();
    let rect = frame_rect(&node.frame);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 52.0,
    };
    push_segmented_control_commands(&mut commands, &node, &rect, &clip, 10, 1.0);
    let ring = commands.iter().find(|c| c.border_color.is_some()).unwrap();
    assert_eq!(ring.background_color, None);
    assert_eq!(ring.border_width, 1.0);
    assert_eq!(ring.frame, tab_paint_rect(&node, &rect));
    let underline = commands.iter().find(|c| c.z_index == 12).unwrap();
    assert!(ring.z_index < underline.z_index);
    let x = (underline.frame.x + underline.frame.width * 0.5) as u32;
    let y = (underline.frame.y + underline.frame.height * 0.5) as u32;
    assert_eq!(
        pixel_at(&bytes, 180, x, y),
        underline.background_color.unwrap()
    );
    node.focus_visible = false;
    let pointer = paint_template_nodes_for_test(180, 52, model_rc(vec![node.clone()]));
    assert_eq!(pixel_at(&pointer, 180, 30, 6), PALETTE.surface_panel);
    node.focus_visible = true;
    node.disabled = true;
    let disabled = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));
    assert_eq!(pixel_at(&disabled, 180, 30, 6), PALETTE.surface_disabled);
}
