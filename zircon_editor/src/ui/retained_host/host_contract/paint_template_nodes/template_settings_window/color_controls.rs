use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{HostControlMetrics, HostMaterialPalette};
use super::super::super::settings_color_editor_geometry::{
    settings_color_channel_frames_within, settings_color_popup_frame_within,
    SETTINGS_COLOR_CHANNEL_COUNT,
};
use super::super::super::settings_window_geometry::SettingsWindowLayout;
use super::super::render_commands::HostPaintCommand;
use super::geometry::inset_rect;

const COLOR_CHANNEL_LABELS: [&str; SETTINGS_COLOR_CHANNEL_COUNT] = ["R", "G", "B", "A"];

#[allow(clippy::too_many_arguments)]
pub(super) fn push_color_swatch(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    rgba: [u8; 4],
    value_text: String,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.surface_inset),
        Some(palette.border),
        metrics.border_width,
        metrics.radius_control,
        opacity,
    ));
    let preview = FrameRect {
        x: rect.x + metrics.gap_s,
        y: rect.y + metrics.gap_s,
        width: (rect.height * 1.4).min(rect.width * 0.42).max(1.0),
        height: (rect.height - metrics.gap_s * 2.0).max(1.0),
    };
    push_alpha_checkerboard(commands, &preview, clip, order + 1, opacity, palette);
    commands.push(HostPaintCommand::quad(
        preview.clone(),
        Some(clip.clone()),
        order + 2,
        Some(rgba),
        Some(palette.border),
        metrics.border_width,
        metrics.radius_control.min(preview.height * 0.2),
        opacity,
    ));
    let text = FrameRect {
        x: preview.x + preview.width + metrics.gap_m,
        y: rect.y,
        width: (rect.x + rect.width - preview.x - preview.width - metrics.gap_m * 2.0).max(1.0),
        height: rect.height,
    };
    commands.push(HostPaintCommand::text(
        text,
        Some(clip.clone()),
        order + 3,
        value_text,
        palette.text,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_color_popup(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    layout: &SettingsWindowLayout,
    bounds: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    if node.settings_editor_open_kind.as_str() != "color" {
        return;
    }
    let Some(setting_row) = usize::try_from(node.settings_editor_open_row).ok() else {
        return;
    };
    let Some(entry) = node.settings_entries.get(setting_row) else {
        return;
    };
    if entry.schema.as_str() != "color" {
        return;
    }
    let resettable = !entry.value_source.is_empty() && entry.value_source.as_str() != "default";
    let control = layout.setting_color_control(setting_row, resettable);
    let Some(popup) = settings_color_popup_frame_within(&control, bounds) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        popup,
        Some(clip.clone()),
        order,
        Some(palette.popup),
        Some(palette.border),
        metrics.border_width,
        metrics.radius_panel,
        opacity,
    ));
    for (channel, label) in COLOR_CHANNEL_LABELS.iter().enumerate() {
        let Some(frames) = settings_color_channel_frames_within(&control, channel, bounds) else {
            break;
        };
        push_color_channel(
            commands,
            &frames.label,
            &frames.decrement,
            &frames.value,
            &frames.increment,
            label,
            entry.color_rgba[channel],
            clip,
            order + 1 + channel as i32 * 3,
            opacity,
            palette,
            metrics,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_color_channel(
    commands: &mut Vec<HostPaintCommand>,
    label_frame: &FrameRect,
    decrement: &FrameRect,
    value: &FrameRect,
    increment: &FrameRect,
    label: &str,
    channel_value: u8,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    commands.push(HostPaintCommand::text(
        inset_rect(label_frame, metrics.gap_s, 0.0),
        Some(clip.clone()),
        order,
        label.to_owned(),
        palette.text_muted,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
    for frame in [decrement, value, increment] {
        commands.push(HostPaintCommand::quad(
            frame.clone(),
            Some(clip.clone()),
            order,
            Some(palette.surface_inset),
            Some(palette.border),
            metrics.border_width,
            metrics.radius_control.min(frame.height * 0.2),
            opacity,
        ));
    }
    for (frame, text) in [
        (decrement, "-".to_owned()),
        (value, channel_value.to_string()),
        (increment, "+".to_owned()),
    ] {
        commands.push(HostPaintCommand::text(
            frame.clone(),
            Some(clip.clone()),
            order + 1,
            text,
            palette.text,
            metrics.font_body,
            metrics.line_height(metrics.font_body),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

pub(super) fn push_alpha_checkerboard(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
) {
    let tile_width = rect.width / 4.0;
    let tile_height = rect.height / 2.0;
    for row in 0..2 {
        for column in 0..4 {
            commands.push(HostPaintCommand::quad(
                FrameRect {
                    x: rect.x + column as f32 * tile_width,
                    y: rect.y + row as f32 * tile_height,
                    width: tile_width,
                    height: tile_height,
                },
                Some(clip.clone()),
                order,
                Some(if (row + column) % 2 == 0 {
                    palette.surface
                } else {
                    palette.surface_inset
                }),
                None,
                0.0,
                0.0,
                opacity,
            ));
        }
    }
}
