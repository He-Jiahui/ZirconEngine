use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_text::measure_runtime_text_width;
use super::super::super::super::paint_theme::METRICS;
use super::super::super::style_selector::WORKBENCH_SEMANTIC_STATUS_SIGNAL_VARIANT;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_status_control_geometry::{
    status_font_size, status_signal_text_rect, workbench_status_metrics,
};
use super::super::super::template_style::text_color;
use super::super::{
    push_status_control_commands, status_control_kind, status_signal_icon_fill,
    status_signal_icon_paint_rect, status_signal_icon_rect, status_signal_text_color,
    status_signal_text_gap, StatusControlKind, StatusSignalKind, PALETTE, STATUS_NO_ERRORS_FILL,
};
use super::support::{changed_pixel_count, pixel_at, status_node};
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn ready_status_item_paints_dot_and_text_without_chip_surface() {
    let bytes = paint_template_nodes_for_test(
        140,
        46,
        model_rc(vec![status_node(
            "WorkbenchStatusReady",
            "Ready",
            72.0,
            46.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 140, 12, 23), PALETTE.success);
    assert_eq!(pixel_at(&bytes, 140, 90, 4), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 140, 42, 14, 40, 18) > 0);
}

#[test]
fn ready_status_item_uses_declared_dot_text_and_gap_style() {
    let mut node = status_node("WorkbenchStatusReady", "Ready", 72.0, 46.0);
    node.layout_offset_x = 0.0;
    node.layout_offset_y = 0.0;
    node.layout_content_offset_x = 8.0;
    node.value_number = 9.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(143, 154, 160);
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(78, 170, 95);

    let icon = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 72.0,
            height: 46.0,
        },
        StatusSignalKind::Ready,
    );

    assert!((icon.x - METRICS.gap_m).abs() < 0.001);
    assert!((icon.y - 19.0).abs() < 0.001);
    assert!((icon.width - METRICS.gap_m).abs() < 0.001);
    assert!((status_signal_text_gap(&node) - 8.0).abs() < 0.001);
    assert_eq!(
        status_signal_text_color(&node, StatusSignalKind::Ready),
        [143, 154, 160, 255]
    );
    assert_eq!(
        status_signal_icon_fill(&node, StatusSignalKind::Ready),
        [78, 170, 95, 255]
    );
}

#[test]
fn errors_status_item_uses_audited_success_icon_fill() {
    let bytes = paint_template_nodes_for_test(
        140,
        46,
        model_rc(vec![status_node(
            "WorkbenchStatusErrors",
            "No Errors",
            92.0,
            46.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 140, 12, 23), STATUS_NO_ERRORS_FILL);
    assert!(changed_pixel_count(&bytes, 140, 46, 14, 58, 18) > 0);
}

#[test]
fn errors_status_item_ignores_legacy_mark_color() {
    let mut node = status_node("WorkbenchStatusErrors", "No Errors", 116.0, 46.0);
    node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 32, 24);

    assert_eq!(
        status_signal_icon_fill(&node, StatusSignalKind::Success),
        STATUS_NO_ERRORS_FILL
    );
}

#[test]
fn semantic_status_signal_variant_uses_central_palette_instead_of_declared_colors() {
    for (control_id, kind, marker, text) in [
        (
            "WorkbenchStatusReady",
            StatusSignalKind::Ready,
            PALETTE.success,
            PALETTE.text,
        ),
        (
            "WorkbenchStatusErrors",
            StatusSignalKind::Success,
            STATUS_NO_ERRORS_FILL,
            PALETTE.text_muted,
        ),
        (
            "WorkbenchStatusWarnings",
            StatusSignalKind::Warning,
            PALETTE.warning,
            PALETTE.text_muted,
        ),
        (
            "WorkbenchStatusMessages",
            StatusSignalKind::Info,
            PALETTE.info,
            PALETTE.text_muted,
        ),
    ] {
        let mut node = status_node(control_id, "Status", 116.0, 28.0);
        node.component_variant = WORKBENCH_SEMANTIC_STATUS_SIGNAL_VARIANT.into();
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(255, 0, 255);
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(0, 255, 0);

        assert_eq!(status_signal_icon_fill(&node, kind), marker);
        assert_eq!(status_signal_text_color(&node, kind), text);
    }
}

#[test]
fn status_signal_item_ignores_legacy_icon_size_overrides() {
    let mut node = status_node("WorkbenchStatusErrors", "No Errors", 116.0, 46.0);
    node.value_number = 21.0;
    node.layout_icon_size = 12.04;
    node.layout_content_offset_y = -3.0;

    let layout = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 116.0,
            height: 46.0,
        },
        StatusSignalKind::Success,
    );
    let paint = status_signal_icon_paint_rect(&node, &layout, StatusSignalKind::Success);

    assert!((layout.x - METRICS.gap_m).abs() < 0.001);
    assert!((layout.y - 19.0).abs() < 0.001);
    assert!((layout.width - METRICS.gap_m).abs() < 0.001);
    assert!((paint.x - METRICS.gap_m).abs() < 0.001);
    assert!((paint.y - 19.0).abs() < 0.001);
    assert!((paint.width - METRICS.gap_m).abs() < 0.001);
}

#[test]
fn warning_status_item_uses_declared_marker_text_and_gap_style() {
    let mut node = status_node("WorkbenchStatusWarnings", "2 Warnings", 96.0, 46.0);
    node.layout_offset_x = 0.0;
    node.layout_offset_y = 0.0;
    node.layout_content_offset_x = 8.0;
    node.layout_content_offset_y = 0.0;
    node.value_number = 21.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(135, 146, 153);
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(242, 195, 86);
    node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 24, 26);
    node.icon_stroke_width = 1.45;

    let icon = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 96.0,
            height: 46.0,
        },
        StatusSignalKind::Warning,
    );

    assert!((icon.x - METRICS.gap_m).abs() < 0.001);
    assert!((icon.y - 19.0).abs() < 0.001);
    assert!((icon.width - METRICS.gap_m).abs() < 0.001);
    assert!((status_signal_text_gap(&node) - 8.0).abs() < 0.001);
    assert_eq!(
        status_signal_text_color(&node, StatusSignalKind::Warning),
        [135, 146, 153, 255]
    );
    assert_eq!(
        status_signal_icon_fill(&node, StatusSignalKind::Warning),
        [242, 195, 86, 255]
    );
}

#[test]
fn messages_status_item_uses_declared_marker_text_and_offset_style() {
    let mut node = status_node("WorkbenchStatusMessages", "0 Messages", 100.0, 46.0);
    node.layout_offset_x = 0.0;
    node.layout_offset_y = 0.0;
    node.layout_content_offset_x = 8.0;
    node.layout_content_offset_y = 0.0;
    node.value_number = 18.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(151, 163, 169);
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(76, 154, 232);

    let icon = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 46.0,
        },
        StatusSignalKind::Info,
    );

    assert!((icon.x - METRICS.gap_m).abs() < 0.001);
    assert!((icon.y - 19.0).abs() < 0.001);
    assert!((icon.width - METRICS.gap_m).abs() < 0.001);
    assert_eq!(
        status_signal_text_color(&node, StatusSignalKind::Info),
        [151, 163, 169, 255]
    );
    assert_eq!(
        status_signal_icon_fill(&node, StatusSignalKind::Info),
        [76, 154, 232, 255]
    );
}

#[test]
fn diagnostic_status_signal_variant_uses_semantic_marker_text_and_compact_spacing() {
    for (level, kind, semantic_color) in [
        ("info", StatusSignalKind::Info, PALETTE.text_muted),
        ("warning", StatusSignalKind::Warning, PALETTE.warning),
        ("error", StatusSignalKind::Error, PALETTE.error),
    ] {
        let mut node = status_node("DiagnosticSeverity", "[Level]", 84.0, 24.0);
        node.component_variant = "diagnostic_signal".into();
        node.validation_level = level.into();
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(255, 0, 255);
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 84.0,
            height: 24.0,
        };
        let icon = status_signal_icon_rect(&node, &rect, kind);

        assert_eq!(
            status_control_kind(&node),
            Some(StatusControlKind::Signal(kind))
        );
        assert!((icon.x - METRICS.gap_s).abs() < 0.001);
        assert!((status_signal_text_gap(&node) - METRICS.gap_s).abs() < 0.001);
        assert_eq!(status_signal_icon_fill(&node, kind), semantic_color);
        assert_eq!(status_signal_text_color(&node, kind), semantic_color);
    }
}

#[test]
fn diagnostic_info_signal_keeps_muted_severity_distinct_from_primary_messages() {
    let mut severity = status_node("DiagnosticSeverity", "[Info]", 84.0, 24.0);
    severity.component_variant = "diagnostic_signal".into();
    severity.validation_level = "info".into();
    let mut message = status_node(
        "DiagnosticMessage",
        "Blend space axes are valid.",
        180.0,
        24.0,
    );
    message.button_style.element.foreground_color = Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
        PALETTE.text[0],
        PALETTE.text[1],
        PALETTE.text[2],
        PALETTE.text[3],
    )));

    let severity_color = status_signal_text_color(&severity, StatusSignalKind::Info);
    let message_color = text_color(&message);

    assert_eq!(severity_color, PALETTE.text_muted);
    assert_eq!(message_color, PALETTE.text);
    assert_ne!(severity_color, message_color);
}

#[test]
fn diagnostic_status_signal_variant_paints_the_central_semantic_marker() {
    for (level, color) in [
        ("info", PALETTE.text_muted),
        ("warning", PALETTE.warning),
        ("error", PALETTE.error),
    ] {
        let mut node = status_node("DiagnosticSeverity", "[Level]", 84.0, 24.0);
        node.component_variant = "diagnostic_signal".into();
        node.validation_level = level.into();
        let bytes = paint_template_nodes_for_test(100, 24, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 100, 8, 12), color);
    }
}

#[test]
fn non_ready_status_items_emit_single_inline_marker_and_text() {
    for (control_id, text) in [
        ("WorkbenchStatusErrors", "No Errors"),
        ("WorkbenchStatusWarnings", "2 Warnings"),
        ("WorkbenchStatusMessages", "0 Messages"),
    ] {
        let mut node = status_node(control_id, text, 132.0, 46.0);
        node.value_number = 21.0;
        node.layout_icon_size = 18.0;
        node.icon_stroke_width = 1.45;
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 132.0,
            height: 46.0,
        };
        let mut commands = Vec::new();
        push_status_control_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

        let marker_commands = commands
            .iter()
            .filter(|command| command.text.is_none())
            .collect::<Vec<_>>();
        let text_commands = commands
            .iter()
            .filter(|command| command.text.as_deref() == Some(text))
            .collect::<Vec<_>>();

        assert_eq!(
            marker_commands.len(),
            1,
            "{control_id} should paint one marker"
        );
        assert_eq!(
            text_commands.len(),
            1,
            "{control_id} should paint one text run"
        );
        assert!((marker_commands[0].frame.width - METRICS.gap_m).abs() < 0.001);
        assert!((marker_commands[0].frame.height - METRICS.gap_m).abs() < 0.001);
        assert!((marker_commands[0].corner_radius - METRICS.gap_m * 0.5).abs() < 0.001);
    }
}

#[test]
fn compact_status_signals_keep_full_runtime_text_inside_their_authored_widths() {
    let metrics = workbench_status_metrics();

    for (control_id, text, width) in [
        ("WorkbenchStatusReady", "Ready", 72.0),
        ("WorkbenchStatusErrors", "No Errors", 92.0),
        ("WorkbenchStatusWarnings", "2 Warnings", 96.0),
        ("WorkbenchStatusMessages", "0 Messages", 100.0),
    ] {
        let mut node = status_node(control_id, text, width, 46.0);
        node.layout_content_offset_x = 8.0;
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width,
            height: 46.0,
        };
        let kind = match control_id {
            "WorkbenchStatusReady" => StatusSignalKind::Ready,
            "WorkbenchStatusErrors" => StatusSignalKind::Success,
            "WorkbenchStatusWarnings" => StatusSignalKind::Warning,
            "WorkbenchStatusMessages" => StatusSignalKind::Info,
            _ => unreachable!("status fixture must use a known signal identity"),
        };
        let icon = status_signal_icon_rect(&node, &rect, kind);
        let text_rect = status_signal_text_rect(&node, &rect, &icon);
        let required_text_width =
            measure_runtime_text_width(text, status_font_size()) + metrics.text_clip_guard;

        assert!(
            (icon.x - rect.x - metrics.signal_icon_left).abs() <= 0.01,
            "{control_id} should use one compact shared inset before its marker: icon={icon:?}, metrics={metrics:?}"
        );
        assert!(
            text_rect.width >= required_text_width,
            "{control_id} should preserve the full Runtime Text label: available={}, required={required_text_width}, rect={text_rect:?}",
            text_rect.width,
        );
    }
}
