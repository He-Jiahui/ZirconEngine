use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{
    FrameRect, HostTextInputFocusData, TemplatePaneNodeData, TemplateSettingEntryData,
    TemplateSettingsCategoryData,
};
use super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};
use super::super::super::settings_window_geometry::SettingsWindowLayout;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::chord_control::push_chord_control;
use super::color_controls::{push_color_popup, push_color_swatch};
use super::enum_controls::{push_enum_control, push_enum_popup};
use super::geometry::inset_rect;
use super::persistence_health::push_persistence_health;
use super::scrollbars::push_preferences_scrollbars;
use super::text_control::push_string_control;
use super::visible_rows::{
    category_visible_rows, intersect_frames, settings_window_visible_rows, valid_frame,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_settings_window_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    order: i32,
    opacity: f32,
) -> bool {
    if node.component_role.as_str() != "settings-window" {
        return false;
    }
    if !node.popup_open {
        return true;
    }
    if !valid_frame(rect) || !valid_frame(clip) {
        return true;
    }

    let palette = current_host_palette();
    let metrics = current_host_metrics();
    let category_count = node.settings_categories.row_count();
    let row_count = node.settings_entries.row_count();
    let layout = SettingsWindowLayout::new(
        rect,
        metrics,
        node.settings_category_scroll_offset,
        category_count,
        node.settings_scroll_offset,
        row_count,
    );

    push_panel(commands, rect, clip, order, opacity, palette, metrics);
    push_title(
        commands,
        node.settings_title.to_string(),
        &layout,
        clip,
        order + 1,
        opacity,
        palette,
        metrics,
    );
    push_persistence_health(
        commands,
        node,
        &layout,
        clip,
        order + 2,
        opacity,
        palette,
        metrics,
    );
    push_sidebar(
        commands,
        &layout,
        clip,
        order + 2,
        opacity,
        palette,
        metrics,
    );

    if let Some(category_list_clip) = intersect_frames(&layout.category_list, clip) {
        for row in category_visible_rows(
            &layout.category_list,
            &category_list_clip,
            category_count,
            &layout,
        ) {
            let Some(category) = node.settings_categories.get(row) else {
                continue;
            };
            push_category(
                commands,
                category,
                &layout.category_row(row),
                &category_list_clip,
                order + 3 + row as i32 * 3,
                opacity,
                palette,
                metrics,
            );
        }
    }

    if let Some(category) = selected_category(node) {
        push_content_heading(
            commands,
            category.label_path.to_string(),
            &layout,
            clip,
            order + 3,
            opacity,
            palette,
            metrics,
        );
    }

    if let Some(setting_list_clip) = intersect_frames(&layout.setting_list, clip) {
        for row in settings_window_visible_rows(
            &layout.setting_list,
            &setting_list_clip,
            row_count,
            &layout,
        ) {
            let Some(entry) = node.settings_entries.get(row) else {
                continue;
            };
            push_setting(
                commands,
                entry,
                &layout,
                row,
                &setting_list_clip,
                text_input_focus,
                order + 5 + row as i32 * 7,
                opacity,
                palette,
                metrics,
            );
        }
    }
    push_preferences_scrollbars(
        commands,
        &layout,
        clip,
        order + 9_000,
        opacity,
        palette,
        metrics,
    );
    push_enum_popup(
        commands,
        node,
        &layout,
        rect,
        clip,
        order + 10_000,
        opacity,
        palette,
        metrics,
    );
    push_color_popup(
        commands,
        node,
        &layout,
        rect,
        clip,
        order + 10_000,
        opacity,
        palette,
        metrics,
    );

    true
}

fn selected_category(node: &TemplatePaneNodeData) -> Option<&TemplateSettingsCategoryData> {
    node.settings_categories.iter().find(|category| {
        category.selected && category.id.as_str() == node.selected_settings_category_id.as_str()
    })
}

fn push_panel(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
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
        Some(palette.popup),
        Some(palette.border),
        metrics.border_width,
        metrics.radius_panel,
        opacity,
    ));
}

fn push_title(
    commands: &mut Vec<HostPaintCommand>,
    title: String,
    layout: &SettingsWindowLayout,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    if title.is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        layout.title.clone(),
        Some(clip.clone()),
        order,
        title,
        palette.text,
        metrics.font_large,
        metrics.line_height(metrics.font_large),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_sidebar(
    commands: &mut Vec<HostPaintCommand>,
    layout: &SettingsWindowLayout,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    commands.push(HostPaintCommand::quad(
        layout.sidebar.clone(),
        Some(clip.clone()),
        order,
        Some(palette.surface_inset),
        Some(palette.separator_soft),
        metrics.border_width,
        0.0,
        opacity,
    ));
}

fn push_category(
    commands: &mut Vec<HostPaintCommand>,
    category: &TemplateSettingsCategoryData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    if category.selected {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(palette.surface_selected),
            None,
            0.0,
            metrics.radius_control,
            opacity,
        ));
        let indicator = FrameRect {
            x: rect.x,
            y: rect.y + metrics.gap_s,
            width: metrics.selection_indicator_width,
            height: (rect.height - metrics.gap_m).max(0.0),
        };
        commands.push(HostPaintCommand::quad(
            indicator,
            Some(clip.clone()),
            order + 1,
            Some(palette.accent),
            None,
            0.0,
            metrics.selection_indicator_width,
            opacity,
        ));
    }
    let depth = category.key_path.matches('/').count() as f32;
    commands.push(HostPaintCommand::text(
        inset_rect(rect, metrics.gap_l + depth * metrics.gap_m, metrics.gap_s),
        Some(clip.clone()),
        order + 2,
        category.label.to_string(),
        if category.selected {
            palette.text
        } else {
            palette.text_muted
        },
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_content_heading(
    commands: &mut Vec<HostPaintCommand>,
    label: String,
    layout: &SettingsWindowLayout,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    if label.is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        layout.content_heading.clone(),
        Some(clip.clone()),
        order,
        label,
        palette.text,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_setting(
    commands: &mut Vec<HostPaintCommand>,
    entry: &TemplateSettingEntryData,
    layout: &SettingsWindowLayout,
    row: usize,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    let rect = layout.setting_row(row);
    let resettable = !entry.plugin_page
        && !entry.value_source.is_empty()
        && entry.value_source.as_str() != "default";
    let value_control = layout.setting_value_control(row, resettable);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.surface),
        Some(if entry.plugin_page {
            palette.accent_soft
        } else {
            palette.separator_soft
        }),
        metrics.border_width,
        metrics.radius_control,
        opacity,
    ));
    let text_x = rect.x + metrics.gap_l;
    let text_width = layout.setting_text_width(row);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: text_x,
            y: rect.y + metrics.gap_m,
            width: text_width,
            height: metrics.line_height(metrics.font_body),
        },
        Some(clip.clone()),
        order + 1,
        entry.label.to_string(),
        palette.text,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
    if !entry.description.is_empty() {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: text_x,
                y: rect.y + metrics.gap_m + metrics.line_height(metrics.font_body),
                width: text_width,
                height: metrics.line_height(metrics.font_small),
            },
            Some(clip.clone()),
            order + 2,
            entry.description.to_string(),
            palette.text_muted,
            metrics.font_small,
            metrics.line_height(metrics.font_small),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
    if entry.schema.as_str() == "bool" && !entry.value_text.is_empty() {
        push_bool_control(
            commands,
            &layout.setting_bool_control(row, resettable),
            entry.value_text.as_str() == "true",
            clip,
            order + 3,
            opacity,
            palette,
            metrics,
        );
    } else if matches!(entry.schema.as_str(), "int" | "float") && !entry.value_text.is_empty() {
        push_numeric_stepper(
            commands,
            &layout.setting_numeric_decrement_control(row, resettable),
            &layout.setting_numeric_value_frame(row, resettable),
            &layout.setting_numeric_increment_control(row, resettable),
            entry.value_text.to_string(),
            clip,
            order + 3,
            opacity,
            palette,
            metrics,
        );
    } else if entry.schema.as_str() == "enum" && !entry.value_text.is_empty() {
        push_enum_control(
            commands,
            &layout.setting_enum_control(row, resettable),
            entry.value_text.to_string(),
            clip,
            order + 3,
            opacity,
            palette,
            metrics,
        );
    } else if entry.schema.as_str() == "color" && !entry.value_text.is_empty() {
        push_color_swatch(
            commands,
            &layout.setting_color_control(row, resettable),
            entry.color_rgba,
            entry.value_text.to_string(),
            clip,
            order + 3,
            opacity,
            palette,
            metrics,
        );
    } else if entry.schema.as_str() == "string" {
        push_string_control(
            commands,
            &value_control,
            entry.key.as_str(),
            entry.value_text.as_str(),
            text_input_focus,
            clip,
            order + 3,
            opacity,
            palette,
            metrics,
        );
    } else if entry.schema.as_str() == "chord" {
        push_chord_control(
            commands,
            &value_control,
            entry.key.as_str(),
            entry.value_text.as_str(),
            text_input_focus,
            clip,
            order + 3,
            opacity,
            palette,
            metrics,
        );
    } else if !entry.value_text.is_empty() {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: value_control.x,
                y: rect.y + metrics.gap_m,
                width: value_control.width,
                height: metrics.line_height(metrics.font_body),
            },
            Some(clip.clone()),
            order + 3,
            entry.value_text.to_string(),
            palette.text,
            metrics.font_body,
            metrics.line_height(metrics.font_body),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
    let metadata = setting_metadata(entry);
    if !metadata.is_empty() {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: value_control.x,
                y: rect.y + metrics.gap_m + metrics.line_height(metrics.font_body),
                width: value_control.width,
                height: metrics.line_height(metrics.font_small),
            },
            Some(clip.clone()),
            order + 3,
            metadata,
            palette.text_muted,
            metrics.font_small,
            metrics.line_height(metrics.font_small),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
    if resettable {
        push_reset_control(
            commands,
            &layout.setting_reset_control(row),
            clip,
            order + 5,
            opacity,
            palette,
            metrics,
        );
    }
    if entry.requires_restart {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + rect.width - metrics.gap_l,
                y: rect.y + rect.height - metrics.gap_l,
                width: metrics.gap_s,
                height: metrics.gap_s,
            },
            Some(clip.clone()),
            order + 6,
            Some(palette.warning),
            None,
            0.0,
            metrics.gap_s,
            opacity,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn push_numeric_stepper(
    commands: &mut Vec<HostPaintCommand>,
    decrement: &FrameRect,
    value: &FrameRect,
    increment: &FrameRect,
    value_text: String,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
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
    for (frame, label) in [(decrement, "-"), (increment, "+")] {
        commands.push(HostPaintCommand::text(
            frame.clone(),
            Some(clip.clone()),
            order + 1,
            label.to_owned(),
            palette.text,
            metrics.font_body,
            metrics.line_height(metrics.font_body),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
    commands.push(HostPaintCommand::text(
        value.clone(),
        Some(clip.clone()),
        order + 1,
        value_text,
        palette.text,
        metrics.font_body,
        metrics.line_height(metrics.font_body),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

#[allow(clippy::too_many_arguments)]
fn push_bool_control(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    checked: bool,
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
        Some(if checked {
            palette.accent
        } else {
            palette.surface_inset
        }),
        Some(if checked {
            palette.accent
        } else {
            palette.border
        }),
        metrics.border_width,
        metrics.radius_control.min(rect.width * 0.2),
        opacity,
    ));
    if checked {
        let inset = metrics.gap_s.max(metrics.border_width);
        let icon = FrameRect {
            x: rect.x + inset,
            y: rect.y + inset,
            width: (rect.width - inset * 2.0).max(1.0),
            height: (rect.height - inset * 2.0).max(1.0),
        };
        push_icon_asset_pixels(
            commands,
            "checkmark",
            &icon,
            clip,
            order + 1,
            Some(palette.text),
            opacity,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_reset_control(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    let inset = metrics.gap_s.max(metrics.border_width);
    let icon = FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: (rect.width - inset * 2.0).max(1.0),
        height: (rect.height - inset * 2.0).max(1.0),
    };
    push_icon_asset_pixels(
        commands,
        "reset",
        &icon,
        clip,
        order,
        Some(palette.text_muted),
        opacity,
    );
}

fn setting_metadata(entry: &TemplateSettingEntryData) -> String {
    [
        entry.scope.as_str(),
        entry.schema.as_str(),
        entry.value_source.as_str(),
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(" / ")
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
