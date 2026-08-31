use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{HostControlMetrics, HostMaterialPalette};
use super::super::super::settings_window_geometry::SettingsWindowLayout;
use super::super::super::template_popup_layout::{
    dropdown_option_popup_frame_within, dropdown_option_row_frame_within,
};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::geometry::inset_rect;

#[allow(clippy::too_many_arguments)]
pub(super) fn push_enum_control(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
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
    commands.push(HostPaintCommand::text(
        inset_rect(rect, metrics.gap_m, 0.0),
        Some(clip.clone()),
        order + 1,
        value_text,
        palette.text,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
    let size = rect.height.min(metrics.control_default_height);
    let icon = FrameRect {
        x: rect.x + rect.width - size,
        y: rect.y,
        width: size,
        height: rect.height,
    };
    push_icon_asset_pixels(
        commands,
        "chevron-down",
        &icon,
        clip,
        order + 2,
        Some(palette.text_muted),
        opacity,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_enum_popup(
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
    if node.settings_editor_open_kind.as_str() != "enum" {
        return;
    }
    let Some(setting_row) = usize::try_from(node.settings_editor_open_row).ok() else {
        return;
    };
    let Some(entry) = node.settings_entries.get(setting_row) else {
        return;
    };
    let resettable = !entry.value_source.is_empty() && entry.value_source.as_str() != "default";
    let control = layout.setting_enum_control(setting_row, resettable);
    let Some(popup) = dropdown_option_popup_frame_within(&control, entry.options.len(), bounds)
    else {
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
    for (row, option) in entry.options.iter().enumerate() {
        let Some(frame) =
            dropdown_option_row_frame_within(&control, entry.options.len(), row, bounds)
        else {
            break;
        };
        if option.as_str() == entry.value_text.as_str() {
            commands.push(HostPaintCommand::quad(
                frame.clone(),
                Some(clip.clone()),
                order + 1,
                Some(palette.surface_selected),
                None,
                0.0,
                metrics.radius_control,
                opacity,
            ));
        }
        commands.push(HostPaintCommand::text(
            inset_rect(&frame, metrics.gap_m, 0.0),
            Some(clip.clone()),
            order + 2,
            option.to_string(),
            palette.text,
            metrics.font_body,
            metrics.line_height(metrics.font_body),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}
