use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::paint_text::measure_runtime_text_width_with_style;
use super::render_commands::HostPaintCommand;
use super::template_node_labels::{template_node_has_label, template_node_label};
use super::template_style::text_color;

mod command;
mod eligibility;
mod geometry;
mod metrics;

use command::{is_paintable_text_slot, push_text_command};
use eligibility::{should_skip_template_text, should_skip_template_text_before_label};
use geometry::text_rect_for_node;
use metrics::node_font_size;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const WELCOME_LABEL_PAINT_DIAGNOSTIC_ENV: &str = "ZIRCON_TRACE_WELCOME_LABEL_PAINT";

struct WelcomeLabelPaintDiagnostic {
    control_id: &'static str,
    expected_text: &'static str,
    observed_text: String,
    text_rect: FrameRect,
    font_size: f32,
    color: [u8; 4],
    opacity: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_text_fallback_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_input_focus: Option<&HostTextInputFocusData>,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
    opacity: f32,
) {
    let diagnostic_target = welcome_label_diagnostic_target(node);
    let diagnostic = diagnostic_target
        .filter(|_| welcome_label_paint_diagnostic_enabled())
        .map(|(control_id, expected_text)| {
            let text_rect = text_rect_for_node(node, rect);
            WelcomeLabelPaintDiagnostic {
                control_id,
                expected_text,
                observed_text: template_node_label(node, text_input_focus),
                font_size: node_font_size(node, text_rect.height),
                color: text_color(node),
                opacity,
                text_rect,
            }
        });
    let command_count_before = commands.len();
    let edit_feedback = if should_skip_template_text_before_label(
        node,
        property_row_text_painted,
        table_row_text_painted,
    ) {
        None
    } else {
        focused_text_edit_feedback(node, rect, clip, order, text_input_focus, opacity)
    };
    let (selection_command, caret_command) =
        edit_feedback.map_or((None, None), |(selection, caret)| (selection, Some(caret)));
    if let Some(selection) = selection_command {
        commands.push(selection);
    }
    let outcome;

    if should_skip_template_text_before_label(
        node,
        property_row_text_painted,
        table_row_text_painted,
    ) {
        outcome = "skipped-before-label";
    } else if !template_node_has_label(node, text_input_focus) {
        outcome = "no-label-text";
    } else {
        let (text_rect, font_size) = diagnostic
            .as_ref()
            .map(|diagnostic| (diagnostic.text_rect.clone(), diagnostic.font_size))
            .unwrap_or_else(|| {
                let text_rect = text_rect_for_node(node, rect);
                let font_size = node_font_size(node, text_rect.height);
                (text_rect, font_size)
            });
        if !is_paintable_text_slot(&text_rect, clip, font_size) {
            outcome = "unpaintable-rect-font-or-clip";
        } else {
            let label = diagnostic
                .as_ref()
                .map(|diagnostic| diagnostic.observed_text.clone())
                .unwrap_or_else(|| template_node_label(node, text_input_focus));
            if should_skip_template_text(
                node,
                &label,
                property_row_text_painted,
                table_row_text_painted,
            ) {
                outcome = "skipped-by-text-policy";
            } else {
                push_text_command(
                    commands,
                    &text_rect,
                    clip,
                    order,
                    label,
                    diagnostic
                        .as_ref()
                        .map_or_else(|| text_color(node), |diagnostic| diagnostic.color),
                    font_size,
                    node_text_paint_style(node),
                    opacity,
                );
                outcome = if commands.len() > command_count_before {
                    "emitted"
                } else {
                    "push-rejected"
                };
            }
        }
    }

    if let Some(caret) = caret_command {
        commands.push(caret);
    }
    if let Some(diagnostic) = diagnostic {
        record_welcome_label_paint_diagnostic(
            diagnostic,
            rect,
            clip,
            outcome,
            commands.len() > command_count_before,
        );
    }
}

fn welcome_label_diagnostic_target(
    node: &TemplatePaneNodeData,
) -> Option<(&'static str, &'static str)> {
    match node.control_id.as_str() {
        "WelcomeProjectNameLabel" => Some(("WelcomeProjectNameLabel", "Project name")),
        "WelcomeLocationLabel" => Some(("WelcomeLocationLabel", "Location")),
        _ => None,
    }
}

fn welcome_label_paint_diagnostic_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var(WELCOME_LABEL_PAINT_DIAGNOSTIC_ENV)
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    })
}

fn record_welcome_label_paint_diagnostic(
    diagnostic: WelcomeLabelPaintDiagnostic,
    node_rect: &FrameRect,
    clip: &FrameRect,
    outcome: &str,
    command_pushed: bool,
) {
    let text_status = if diagnostic.observed_text == diagnostic.expected_text {
        format!("exact({:?})", diagnostic.expected_text)
    } else if diagnostic.observed_text.trim().is_empty() {
        format!("empty(expected={:?})", diagnostic.expected_text)
    } else {
        format!(
            "unexpected-length-{}(expected={:?})",
            diagnostic.observed_text.chars().count(),
            diagnostic.expected_text
        )
    };
    let signature = format!(
        "{}|{}|{}|{}|{}|{:.2}|{:?}|{:.2}|{}|{}",
        diagnostic.control_id,
        text_status,
        rect_signature(node_rect),
        rect_signature(&diagnostic.text_rect),
        rect_signature(clip),
        diagnostic.font_size,
        diagnostic.color,
        diagnostic.opacity,
        outcome,
        command_pushed
    );
    static REPORTED_DIAGNOSTICS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let reported = REPORTED_DIAGNOSTICS.get_or_init(|| Mutex::new(HashSet::new()));
    let Ok(mut reported) = reported.lock() else {
        return;
    };
    if !reported.insert(signature) {
        return;
    }

    eprintln!(
        "[welcome-label-paint] control_id={} text={} node_rect={} text_rect={} clip={} font_size={:.2} color_rgba={:?} opacity={:.2} outcome={} command_pushed={}",
        diagnostic.control_id,
        text_status,
        rect_signature(node_rect),
        rect_signature(&diagnostic.text_rect),
        rect_signature(clip),
        diagnostic.font_size,
        diagnostic.color,
        diagnostic.opacity,
        outcome,
        command_pushed
    );
}

fn rect_signature(rect: &FrameRect) -> String {
    format!(
        "({:.1},{:.1},{:.1},{:.1})",
        rect.x, rect.y, rect.width, rect.height
    )
}

fn focused_text_edit_feedback(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_input_focus: Option<&HostTextInputFocusData>,
    opacity: f32,
) -> Option<(Option<HostPaintCommand>, HostPaintCommand)> {
    let focus = text_input_focus.filter(|focus| {
        focus.accepts_text_input() && focus.control_id.as_str() == node.control_id.as_str()
    })?;
    let text_rect = text_rect_for_node(node, rect);
    let field_clip = intersect_text_rect_with_clip(&text_rect, clip)?;
    let font_size = node_font_size(node, text_rect.height);
    if !is_paintable_text_slot(&text_rect, &field_clip, font_size) {
        return None;
    }

    let style = node_text_paint_style(node);
    focused_text_edit_feedback_in_slot(
        focus, &text_rect, field_clip, font_size, style, order, opacity,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn focused_text_edit_feedback_for_text_command(
    node: &TemplatePaneNodeData,
    text_command: &HostPaintCommand,
    text_input_focus: Option<&HostTextInputFocusData>,
    opacity: f32,
) -> Option<(Option<HostPaintCommand>, HostPaintCommand)> {
    let focus = text_input_focus.filter(|focus| {
        focus.accepts_text_input() && focus.control_id.as_str() == node.control_id.as_str()
    })?;
    let text_rect = &text_command.frame;
    let clip = text_command.clip_frame.as_ref().unwrap_or(text_rect);
    let field_clip = intersect_text_rect_with_clip(text_rect, clip)?;
    if !is_paintable_text_slot(text_rect, &field_clip, text_command.font_size) {
        return None;
    }
    // Stay above the field surface, including specialized text at surface + 1.
    let order = text_command.z_index;
    focused_text_edit_feedback_in_slot(
        focus,
        text_rect,
        field_clip,
        text_command.font_size,
        text_command.text_style,
        order,
        opacity,
    )
}

fn focused_text_edit_feedback_in_slot(
    focus: &HostTextInputFocusData,
    text_rect: &FrameRect,
    field_clip: FrameRect,
    font_size: f32,
    style: UiTextRunPaintStyle,
    order: i32,
    opacity: f32,
) -> Option<(Option<HostPaintCommand>, HostPaintCommand)> {
    let text = focus.value_text.as_str();
    let selection = focus.selected_scalar_range().and_then(|(start, end)| {
        let start_width = measured_prefix_width(text, start, font_size, style);
        let end_width = measured_prefix_width(text, end, font_size, style);
        let left = (text_rect.x + start_width).clamp(text_rect.x, text_rect.right());
        let right = (text_rect.x + end_width).clamp(text_rect.x, text_rect.right());
        (right > left).then(|| {
            HostPaintCommand::quad(
                FrameRect {
                    x: left,
                    y: text_rect.y,
                    width: right - left,
                    height: text_rect.height,
                },
                Some(field_clip.clone()),
                order.saturating_sub(1),
                Some([77, 137, 255, 102]),
                None,
                0.0,
                0.0,
                opacity,
            )
        })
    });

    let caret_width = 1.0_f32.min(text_rect.width);
    let caret_offset =
        measured_prefix_width(text, focus.resolved_caret_scalar_offset(), font_size, style);
    let caret_x = (text_rect.x + caret_offset).clamp(text_rect.x, text_rect.right() - caret_width);
    let caret = HostPaintCommand::quad(
        FrameRect {
            x: caret_x,
            y: text_rect.y,
            width: caret_width,
            height: text_rect.height,
        },
        Some(field_clip),
        order.saturating_add(1),
        Some([232, 238, 247, 255]),
        None,
        0.0,
        0.0,
        opacity,
    );
    Some((selection, caret))
}

fn measured_prefix_width(
    text: &str,
    scalar_offset: usize,
    font_size: f32,
    style: UiTextRunPaintStyle,
) -> f32 {
    let prefix = text
        .char_indices()
        .nth(scalar_offset)
        .map_or(text, |(byte_offset, _)| &text[..byte_offset]);
    let width = measure_runtime_text_width_with_style(prefix, font_size, style);
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

fn intersect_text_rect_with_clip(text_rect: &FrameRect, clip: &FrameRect) -> Option<FrameRect> {
    let left = text_rect.x.max(clip.x);
    let top = text_rect.y.max(clip.y);
    let right = text_rect.right().min(clip.right());
    let bottom = text_rect.bottom().min(clip.bottom());
    (right > left && bottom > top).then_some(FrameRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

fn node_text_paint_style(node: &TemplatePaneNodeData) -> UiTextRunPaintStyle {
    UiTextRunPaintStyle {
        code: node
            .component_variant
            .split_whitespace()
            .any(|variant| variant.eq_ignore_ascii_case("code")),
        ..UiTextRunPaintStyle::default()
    }
}

#[cfg(test)]
#[path = "tests/template_node_text.rs"]
mod tests;
