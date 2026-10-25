use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn compact_icon_text_uses_shared_caption_size_instead_of_instance_font_size() {
    let node = TemplatePaneNodeData {
        component_variant: "code compact_icon_text".into(),
        font_size: 24.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        width: 54.0,
        height: 28.0,
        ..FrameRect::default()
    };

    let metrics = button_content_metrics_from_host(METRICS);
    assert_eq!(
        button_label_font_size_from_metrics(&node, &rect, metrics),
        METRICS.font_small
    );
}

#[test]
fn explicit_button_font_sizes_follow_host_scale_and_fallback_stays_physical() {
    let node = TemplatePaneNodeData {
        font_size: 12.0,
        ..TemplatePaneNodeData::default()
    };
    let fallback_node = TemplatePaneNodeData::default();
    let rect = FrameRect {
        width: 120.0,
        height: 40.0,
        ..FrameRect::default()
    };
    let high_dpi_metrics = HostControlMetrics {
        scale_factor: 1.5,
        font_small: METRICS.font_small * 1.5,
        font_body: METRICS.font_body * 1.5,
        ..METRICS
    };

    assert_eq!(
        button_label_font_size_from_metrics(
            &node,
            &rect,
            button_content_metrics_from_host(METRICS),
        ),
        12.0
    );
    let high_dpi_button_metrics = button_content_metrics_from_host(high_dpi_metrics);
    assert_eq!(
        button_label_font_size_from_metrics(&node, &rect, high_dpi_button_metrics),
        18.0
    );
    assert_eq!(
        button_label_font_size_from_metrics(&fallback_node, &rect, high_dpi_button_metrics),
        high_dpi_metrics.font_body,
        "host metric fallback is already physical and must not be scaled again"
    );
}

#[test]
fn button_label_font_size_shrinks_only_when_the_node_declares_shrink_overflow() {
    let node = TemplatePaneNodeData {
        overflow: "shrink".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        width: 72.0,
        height: METRICS.control_default_height,
        ..FrameRect::default()
    };
    let label = "Disabled";
    let text_style = UiTextRunPaintStyle::default();
    let base_font_size = button_label_font_size(&node, &rect);
    let fitted_font_size =
        button_label_font_size_for_slot(&node, &rect, label, text_style, 0.0, 0.0);
    let available_ink_width =
        (max_label_slot_width(&node, &rect) - button_content_metrics().text_clip_guard).max(0.0);

    assert!(fitted_font_size < base_font_size);
    assert!(
        measured_label_ink_width(label, fitted_font_size, text_style) <= available_ink_width + 0.01
    );
}

#[test]
fn button_label_font_size_preserves_elide_overflow_at_the_declared_size() {
    let node = TemplatePaneNodeData {
        overflow: "elide".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        width: 72.0,
        height: METRICS.control_default_height,
        ..FrameRect::default()
    };
    let text_style = UiTextRunPaintStyle::default();

    assert_eq!(
        button_label_font_size_for_slot(&node, &rect, "Disabled", text_style, 0.0, 0.0,),
        button_label_font_size(&node, &rect),
    );
}

#[test]
fn authored_zero_and_asymmetric_button_insets_control_text_budget_and_center() {
    let node = TemplatePaneNodeData {
        authored_field_padding: [Some(0.0), Some(7.0), Some(4.0), Some(4.0)],
        ..Default::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 90.0,
        height: 26.0,
    };
    assert_eq!(max_label_slot_width(&node, &rect), 83.0);
    let layout = super::super::layout::button_content_layout(&node, &rect, 0.0, 0.0, 20.0);
    assert_eq!(layout.start_x, 41.5);
    let vertical = button_vertical_content_frame(&node, &rect);
    assert_eq!((vertical.y, vertical.height), (24.0, 18.0));
    let high = button_content_metrics_from_host(HostControlMetrics {
        scale_factor: 1.5,
        ..METRICS
    });
    assert_eq!(
        button_content_padding_from_metrics(&node, high),
        [0.0, 10.5, 6.0, 6.0]
    );
}

#[test]
fn invalid_button_padding_uses_host_defaults_and_overlarge_padding_clamps_content() {
    let node = TemplatePaneNodeData {
        authored_field_padding: [Some(-1.0), Some(f32::NAN), Some(f32::INFINITY), None],
        ..Default::default()
    };
    let metrics = button_content_metrics_from_host(METRICS);
    assert_eq!(
        button_content_padding_from_metrics(&node, metrics),
        [METRICS.button_pad_x, METRICS.button_pad_x, 0.0, 0.0]
    );
    let empty = TemplatePaneNodeData {
        authored_field_padding: [Some(100.0); 4],
        ..Default::default()
    };
    let rect = FrameRect {
        width: 40.0,
        height: 26.0,
        ..Default::default()
    };
    assert_eq!(max_label_slot_width(&empty, &rect), 0.0);
    assert_eq!(button_vertical_content_frame(&empty, &rect).height, 0.0);
}

#[test]
fn button_labels_use_authored_line_box_before_the_host_default() {
    let mut node = TemplatePaneNodeData {
        line_height_ratio: 1.2,
        ..Default::default()
    };
    assert_eq!(button_label_line_height(&node, 13.333333), 16.0);
    node.line_height = 18.0;
    assert_eq!(button_label_line_height(&node, 13.333333), 18.0);
}

#[test]
fn trailing_button_glyph_obeys_authored_padding_and_keeps_default_geometry() {
    use super::super::glyph::trailing_glyph_rect;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_button_glyphs::button_icon_size;
    use crate::ui::retained_host::host_contract::paint_theme::{
        current_host_metrics, enter_host_paint_theme_scope,
        host_paint_theme_snapshot_at_scale_for_test,
    };
    for scale in [1.0, 1.5] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0 * scale,
            height: 40.0 * scale,
        };
        let default = TemplatePaneNodeData::default();
        let icon_size = button_icon_size();
        let glyph = trailing_glyph_rect(&default, &rect).expect("default trailing glyph");
        assert_eq!(
            glyph.x,
            rect.right() - current_host_metrics().button_pad_x - icon_size
        );
        let narrow = FrameRect {
            width: current_host_metrics().button_pad_x + icon_size,
            ..rect.clone()
        };
        assert!(
            trailing_glyph_rect(&default, &narrow).is_some(),
            "keep preexisting narrow default arrows"
        );
        let zero = TemplatePaneNodeData {
            authored_field_padding: [Some(0.0); 4],
            ..Default::default()
        };
        assert_eq!(
            trailing_glyph_rect(&zero, &rect)
                .expect("zero padding glyph")
                .right(),
            rect.right()
        );
        let authored = TemplatePaneNodeData {
            authored_field_padding: [Some(3.0), Some(30.0), Some(4.0), Some(4.0)],
            ..Default::default()
        };
        let vertical = button_vertical_content_frame(&authored, &rect);
        let glyph = trailing_glyph_rect(&authored, &vertical).expect("authored trailing glyph");
        assert_eq!(glyph.right(), rect.right() - 30.0 * scale);
        assert!(glyph.x >= rect.x + 3.0 * scale);
        assert!(glyph.y >= rect.y + 4.0 * scale);
        assert!(glyph.bottom() <= rect.bottom() - 4.0 * scale);
        let excessive = TemplatePaneNodeData {
            authored_field_padding: [Some(0.0), Some(100.0), None, None],
            ..Default::default()
        };
        assert!(trailing_glyph_rect(&excessive, &rect).is_none());
        let invalid = TemplatePaneNodeData {
            authored_field_padding: [Some(-1.0), Some(f32::NAN), None, None],
            ..Default::default()
        };
        assert_eq!(
            trailing_glyph_rect(&invalid, &rect),
            trailing_glyph_rect(&default, &rect)
        );
    }
}

#[test]
fn trailing_button_glyph_emits_no_commands_when_authored_padding_leaves_no_content() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test,
    };
    let _scope = enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(1.0));
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchDropdownButton".into(),
        role: "Button".into(),
        button_variant: "secondary".into(),
        authored_field_padding: [Some(0.0), Some(100.0), None, None],
        ..Default::default()
    };
    let rect = FrameRect {
        width: 100.0,
        height: 40.0,
        ..Default::default()
    };
    let mut commands = Vec::new();
    super::super::entry::push_button_content(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        WorkbenchButtonKind::Secondary,
        1.0,
    );
    assert!(
        commands.is_empty(),
        "an invisible trailing glyph must not produce paint commands"
    );
}

#[test]
fn authored_leading_button_icon_uses_mounted_size_for_glyph_and_reserved_lane() {
    use super::super::glyph::{button_glyph_width, leading_glyph_rect};
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_button_glyphs::{
        button_icon_size, ButtonGlyph,
    };
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test,
    };

    for scale in [1.0, 1.25, 1.5] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 160.0 * scale,
            height: 32.0 * scale,
        };
        for declared in [8.0 * scale, 20.0 * scale, 24.0 * scale] {
            let node = TemplatePaneNodeData {
                layout_icon_size: declared,
                icon_name: "toolbar/settings.svg".into(),
                ..Default::default()
            };
            for glyph in [ButtonGlyph::None, ButtonGlyph::Plus] {
                let frame = leading_glyph_rect(&node, &rect, rect.x + 10.0 * scale);
                assert_eq!((frame.width, frame.height), (declared, declared));
                assert_eq!(frame.y + frame.height / 2.0, rect.y + rect.height / 2.0);
                assert_eq!(button_glyph_width(&node, glyph, false), declared);
                assert_eq!(
                    button_glyph_width(&node, glyph, true),
                    declared + button_icon_gap(&node)
                );
            }
        }
        for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let node = TemplatePaneNodeData {
                layout_icon_size: invalid,
                ..Default::default()
            };
            assert_eq!(
                leading_glyph_rect(&node, &rect, rect.x).width,
                button_icon_size()
            );
            assert_eq!(
                button_glyph_width(&node, ButtonGlyph::Plus, false),
                button_icon_size()
            );
            assert_eq!(button_glyph_width(&node, ButtonGlyph::None, true), 0.0);
        }
    }
}

#[test]
fn authored_leading_button_icon_changes_painted_label_lane_without_moving_its_line_center() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test,
    };

    for scale in [1.0, 1.25, 1.5] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 200.0 * scale,
            height: 32.0 * scale,
        };
        // Asset-backed Settings and the built-in Add glyph use the same lane.
        for (id, asset) in [
            ("OpenSettingsWindow", "toolbar/settings.svg"),
            ("WorkbenchAdd", ""),
        ] {
            let mut label_frames = Vec::new();
            for logical_icon_size in [12.0, 20.0] {
                let node = TemplatePaneNodeData {
                    control_id: id.into(),
                    role: "Button".into(),
                    button_variant: "secondary".into(),
                    text: "Settings".into(),
                    icon_name: asset.into(),
                    layout_icon_size: logical_icon_size * scale,
                    ..Default::default()
                };
                let mut commands = Vec::new();
                super::super::entry::push_button_content(
                    &mut commands,
                    &node,
                    &rect,
                    &rect,
                    0,
                    WorkbenchButtonKind::Secondary,
                    1.0,
                );
                let label = commands
                    .iter()
                    .find(|command| command.text.as_deref() == Some("Settings"))
                    .expect("declared icon size must retain the editable label's paint command");
                assert_eq!(
                    label.frame.y + label.frame.height / 2.0,
                    rect.y + rect.height / 2.0
                );
                assert!(label.frame.right() <= rect.right());
                label_frames.push(label.frame.clone());
            }
            // The whole icon/label group stays centered; half the 8-DIP icon
            // increase moves the label start, preserving its width and baseline.
            assert!((label_frames[1].x - label_frames[0].x - 4.0 * scale).abs() < 0.001);
            assert_eq!(label_frames[0].width, label_frames[1].width);
            assert_eq!(label_frames[0].y, label_frames[1].y);
        }
    }
}
