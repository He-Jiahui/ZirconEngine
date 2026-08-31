use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::frame_geometry::contains_point;
use super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::super::settings_color_editor_geometry::{
    settings_color_channel_frames_within, settings_color_popup_frame_within,
    SETTINGS_COLOR_CHANNEL_COUNT,
};
use super::super::super::super::settings_window_geometry::SettingsWindowLayout;
use super::super::super::super::template_popup_layout::{
    dropdown_option_popup_frame_within, dropdown_option_row_frame_within,
};
use super::super::TemplateNodePointerMoveKind;
use super::hit::TemplatePopupRowTarget;
use crate::ui::settings::{
    SETTINGS_CAPTURE_CHORD_ACTION_ID, SETTINGS_CATEGORY_CHANGED_ACTION_ID,
    SETTINGS_COMMIT_CHORD_ACTION_ID, SETTINGS_COMMIT_STRING_ACTION_ID,
    SETTINGS_DECREMENT_COLOR_ALPHA_ACTION_ID, SETTINGS_DECREMENT_COLOR_BLUE_ACTION_ID,
    SETTINGS_DECREMENT_COLOR_GREEN_ACTION_ID, SETTINGS_DECREMENT_COLOR_RED_ACTION_ID,
    SETTINGS_DECREMENT_NUMBER_ACTION_ID, SETTINGS_EDIT_STRING_ACTION_ID,
    SETTINGS_INCREMENT_COLOR_ALPHA_ACTION_ID, SETTINGS_INCREMENT_COLOR_BLUE_ACTION_ID,
    SETTINGS_INCREMENT_COLOR_GREEN_ACTION_ID, SETTINGS_INCREMENT_COLOR_RED_ACTION_ID,
    SETTINGS_INCREMENT_NUMBER_ACTION_ID, SETTINGS_OPEN_COLOR_ACTION_ID,
    SETTINGS_OPEN_ENUM_ACTION_ID, SETTINGS_RESET_OVERRIDE_ACTION_ID,
    SETTINGS_RETRY_PERSISTENCE_ACTION_ID, SETTINGS_SELECT_ENUM_ACTION_ID,
    SETTINGS_TOGGLE_BOOL_ACTION_ID,
};

const COLOR_CHANNEL_ACTIONS: [(&str, &str); SETTINGS_COLOR_CHANNEL_COUNT] = [
    (
        SETTINGS_DECREMENT_COLOR_RED_ACTION_ID,
        SETTINGS_INCREMENT_COLOR_RED_ACTION_ID,
    ),
    (
        SETTINGS_DECREMENT_COLOR_GREEN_ACTION_ID,
        SETTINGS_INCREMENT_COLOR_GREEN_ACTION_ID,
    ),
    (
        SETTINGS_DECREMENT_COLOR_BLUE_ACTION_ID,
        SETTINGS_INCREMENT_COLOR_BLUE_ACTION_ID,
    ),
    (
        SETTINGS_DECREMENT_COLOR_ALPHA_ACTION_ID,
        SETTINGS_INCREMENT_COLOR_ALPHA_ACTION_ID,
    ),
];

pub(super) fn hit_test_settings_window_target<'a>(
    node: &'a TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowTarget<'a>> {
    if node.component_role.as_str() != "settings-window" {
        return None;
    }
    let frame = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    if !contains_point(&frame, x, y) {
        return None;
    }

    let setting_row_count = node.settings_entries.row_count();
    let layout = SettingsWindowLayout::new(
        &frame,
        current_host_metrics(),
        node.settings_category_scroll_offset,
        node.settings_categories.row_count(),
        node.settings_scroll_offset,
        setting_row_count,
    );
    if !node.settings_persistence_retry_scope.is_empty()
        && contains_point(&layout.persistence_retry, x, y)
    {
        return Some(TemplatePopupRowTarget::Hit {
            kind: TemplateNodePointerMoveKind::Option,
            action_id: SETTINGS_RETRY_PERSISTENCE_ACTION_ID,
            value_text: node.settings_persistence_retry_scope.as_str(),
            frame: layout.persistence_retry.clone(),
        });
    }
    if let Some(target) = hit_test_open_enum(node, &layout, &frame, x, y) {
        return Some(target);
    }
    if let Some(target) = hit_test_open_color(node, &layout, &frame, x, y) {
        return Some(target);
    }
    if contains_point(&layout.category_list, x, y) {
        return Some(hit_test_category(node, &layout, x, y));
    }
    if contains_point(&layout.setting_list, x, y) {
        return Some(hit_test_setting(node, &layout, x, y));
    }
    Some(TemplatePopupRowTarget::Blocked)
}

fn hit_test_category<'a>(
    node: &'a TemplatePaneNodeData,
    layout: &SettingsWindowLayout,
    x: f32,
    y: f32,
) -> TemplatePopupRowTarget<'a> {
    let Some((row, category)) = layout
        .category_row_index_at(y, node.settings_categories.row_count())
        .and_then(|row| {
            node.settings_categories
                .get(row)
                .map(|category| (row, category))
        })
    else {
        return TemplatePopupRowTarget::Blocked;
    };
    let row_frame = layout.category_row(row);
    if !contains_point(&row_frame, x, y) {
        return TemplatePopupRowTarget::Blocked;
    }
    TemplatePopupRowTarget::Hit {
        kind: TemplateNodePointerMoveKind::Option,
        action_id: SETTINGS_CATEGORY_CHANGED_ACTION_ID,
        value_text: category.id.as_str(),
        frame: row_frame,
    }
}

fn hit_test_setting<'a>(
    node: &'a TemplatePaneNodeData,
    layout: &SettingsWindowLayout,
    x: f32,
    y: f32,
) -> TemplatePopupRowTarget<'a> {
    let Some((row, entry)) = layout
        .setting_row_index_at(y, node.settings_entries.row_count())
        .and_then(|row| node.settings_entries.get(row).map(|entry| (row, entry)))
    else {
        return TemplatePopupRowTarget::Blocked;
    };
    if entry.plugin_page {
        return TemplatePopupRowTarget::Blocked;
    }
    let resettable = !entry.value_source.is_empty() && entry.value_source.as_str() != "default";
    if resettable {
        let reset = layout.setting_reset_control(row);
        if contains_point(&reset, x, y) {
            return TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: SETTINGS_RESET_OVERRIDE_ACTION_ID,
                value_text: entry.key.as_str(),
                frame: reset,
            };
        }
    }
    if entry.schema.as_str() == "bool" {
        let toggle = layout.setting_bool_control(row, resettable);
        if contains_point(&toggle, x, y) {
            return TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: SETTINGS_TOGGLE_BOOL_ACTION_ID,
                value_text: entry.key.as_str(),
                frame: toggle,
            };
        }
    } else if matches!(entry.schema.as_str(), "int" | "float") {
        let decrement = layout.setting_numeric_decrement_control(row, resettable);
        if contains_point(&decrement, x, y) {
            return TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: SETTINGS_DECREMENT_NUMBER_ACTION_ID,
                value_text: entry.key.as_str(),
                frame: decrement,
            };
        }
        let increment = layout.setting_numeric_increment_control(row, resettable);
        if contains_point(&increment, x, y) {
            return TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: SETTINGS_INCREMENT_NUMBER_ACTION_ID,
                value_text: entry.key.as_str(),
                frame: increment,
            };
        }
    } else if entry.schema.as_str() == "enum" && !entry.options.is_empty() {
        let control = layout.setting_enum_control(row, resettable);
        if contains_point(&control, x, y) {
            return TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: SETTINGS_OPEN_ENUM_ACTION_ID,
                value_text: entry.key.as_str(),
                frame: control,
            };
        }
    } else if entry.schema.as_str() == "color" {
        let control = layout.setting_color_control(row, resettable);
        if contains_point(&control, x, y) {
            return TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: SETTINGS_OPEN_COLOR_ACTION_ID,
                value_text: entry.key.as_str(),
                frame: control,
            };
        }
    } else if entry.schema.as_str() == "string" {
        let control = layout.setting_value_control(row, resettable);
        if contains_point(&control, x, y) {
            return TemplatePopupRowTarget::TextInput {
                control_id: entry.key.as_str(),
                edit_action_id: SETTINGS_EDIT_STRING_ACTION_ID,
                commit_action_id: SETTINGS_COMMIT_STRING_ACTION_ID,
                value_text: entry.value_text.as_str(),
                frame: control,
            };
        }
    } else if entry.schema.as_str() == "chord" {
        let control = layout.setting_value_control(row, resettable);
        if contains_point(&control, x, y) {
            return TemplatePopupRowTarget::ChordInput {
                control_id: entry.key.as_str(),
                capture_action_id: SETTINGS_CAPTURE_CHORD_ACTION_ID,
                commit_action_id: SETTINGS_COMMIT_CHORD_ACTION_ID,
                value_text: entry.value_text.as_str(),
                frame: control,
            };
        }
    }
    TemplatePopupRowTarget::Blocked
}

fn hit_test_open_enum<'a>(
    node: &'a TemplatePaneNodeData,
    layout: &SettingsWindowLayout,
    bounds: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowTarget<'a>> {
    if node.settings_editor_open_kind.as_str() != "enum" {
        return None;
    }
    let setting_row = usize::try_from(node.settings_editor_open_row).ok()?;
    let entry = node.settings_entries.get(setting_row)?;
    let resettable = !entry.value_source.is_empty() && entry.value_source.as_str() != "default";
    let control = layout.setting_enum_control(setting_row, resettable);
    let popup = dropdown_option_popup_frame_within(&control, entry.options.len(), bounds)?;
    if !contains_point(&popup, x, y) {
        return None;
    }
    let Some(first_row) =
        dropdown_option_row_frame_within(&control, entry.options.len(), 0, bounds)
    else {
        return Some(TemplatePopupRowTarget::Blocked);
    };
    let row_height = first_row.height;
    let row = ((y - popup.y) / row_height).floor().max(0.0) as usize;
    let Some(option) = entry.options.get(row) else {
        return Some(TemplatePopupRowTarget::Blocked);
    };
    let Some(frame) = dropdown_option_row_frame_within(&control, entry.options.len(), row, bounds)
    else {
        return Some(TemplatePopupRowTarget::Blocked);
    };
    Some(TemplatePopupRowTarget::Hit {
        kind: TemplateNodePointerMoveKind::Option,
        action_id: SETTINGS_SELECT_ENUM_ACTION_ID,
        value_text: option.as_str(),
        frame,
    })
}

fn hit_test_open_color<'a>(
    node: &'a TemplatePaneNodeData,
    layout: &SettingsWindowLayout,
    bounds: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowTarget<'a>> {
    if node.settings_editor_open_kind.as_str() != "color" {
        return None;
    }
    let setting_row = usize::try_from(node.settings_editor_open_row).ok()?;
    let entry = node.settings_entries.get(setting_row)?;
    if entry.schema.as_str() != "color" {
        return None;
    }
    let resettable = !entry.value_source.is_empty() && entry.value_source.as_str() != "default";
    let control = layout.setting_color_control(setting_row, resettable);
    let popup = settings_color_popup_frame_within(&control, bounds)?;
    if !contains_point(&popup, x, y) {
        return None;
    }
    for (channel, (decrement_action, increment_action)) in
        COLOR_CHANNEL_ACTIONS.iter().copied().enumerate()
    {
        let Some(frames) = settings_color_channel_frames_within(&control, channel, bounds) else {
            break;
        };
        if contains_point(&frames.decrement, x, y) {
            return Some(TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: decrement_action,
                value_text: entry.key.as_str(),
                frame: frames.decrement,
            });
        }
        if contains_point(&frames.increment, x, y) {
            return Some(TemplatePopupRowTarget::Hit {
                kind: TemplateNodePointerMoveKind::Option,
                action_id: increment_action,
                value_text: entry.key.as_str(),
                frame: frames.increment,
            });
        }
    }
    Some(TemplatePopupRowTarget::Blocked)
}
