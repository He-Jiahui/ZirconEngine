use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::popup_position::{
    PopupPlacement, anchored_popup_frame, has_popup_position_metadata, popup_anchor_frame,
    popup_layout_bounds,
};
use super::popup_rows::{
    PopupAttributeIdSet, PopupRowPaintState, menu_row_height, popup_base_z, popup_row_frame,
    popup_rows_height, push_popup_background, push_popup_row_label, push_popup_row_surface,
    push_popup_separator,
};

pub(super) fn popup_menu_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    frame: UiFrame,
    anchor_frame: Option<UiFrame>,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !is_open_context_menu(metadata) {
        return Vec::new();
    }
    let Some(anchor_frame) = anchor_frame else {
        return Vec::new();
    };
    let items = menu_items(metadata);
    if items.is_empty() {
        return Vec::new();
    }
    let item_count = items.len();

    let base_z = popup_base_z(z_index);
    let Some(row_height) = menu_row_height(metadata, frame, item_count) else {
        return Vec::new();
    };
    let popup_frame = menu_popup_frame(
        metadata,
        frame,
        anchor_frame,
        clip_frame,
        row_height,
        item_count,
    );
    let mut commands = Vec::with_capacity(item_count.saturating_mul(3).saturating_add(3));
    let render_clip = if has_popup_position_metadata(metadata) {
        popup_layout_bounds(frame, clip_frame)
    } else {
        clip_frame
    };
    push_popup_background(
        &mut commands,
        node_id,
        metadata,
        popup_frame,
        render_clip,
        base_z,
        opacity,
    );

    for (row, item) in items.into_iter().enumerate() {
        let Some(row_frame) = popup_row_frame(metadata, popup_frame, item_count, row) else {
            continue;
        };
        let row_z = base_z.saturating_add(1 + row as i32);
        if item.separator {
            push_popup_separator(
                &mut commands,
                node_id,
                row_frame,
                render_clip,
                row_z,
                opacity,
            );
            continue;
        }
        let row_state = item.paint_state();
        let text_color = item.text_color(row_state);
        push_popup_row_surface(
            &mut commands,
            node_id,
            row_frame,
            render_clip,
            row_z,
            row_state,
            opacity,
        );
        push_popup_row_label(
            &mut commands,
            node_id,
            row_frame,
            render_clip,
            row_z.saturating_add(2),
            item.label,
            text_color,
            row_state,
            opacity,
        );
    }

    commands
}

pub(super) fn popup_menu_may_emit_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    is_open_context_menu(metadata)
        && metadata
            .attributes
            .get("menu_items")
            .or_else(|| metadata.attributes.get("options"))
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty())
}

fn menu_popup_frame(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    anchor_frame: UiFrame,
    clip_frame: Option<UiFrame>,
    row_height: f32,
    row_count: usize,
) -> UiFrame {
    if !has_popup_position_metadata(metadata) {
        return frame;
    }
    let anchor_frame = popup_anchor_frame(metadata, anchor_frame);
    let bounds = popup_layout_bounds(frame, clip_frame);
    anchored_popup_frame(
        metadata,
        anchor_frame,
        frame.width.max(1.0),
        popup_rows_height(metadata, row_count, row_height)
            .unwrap_or(frame.height)
            .max(frame.height),
        bounds,
        PopupPlacement::BottomStart,
        4.0,
    )
    .unwrap_or(frame)
}

fn is_open_context_menu(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "ContextActionMenu" | "ContextMenu" | "PopupMenu" | "MenuPopup"
    ) && bool_attribute(metadata, "popup_open").or_else(|| bool_attribute(metadata, "open"))
        == Some(true)
}

fn menu_items<'a>(metadata: &'a UiTemplateNodeMetadata) -> Vec<RuntimePopupMenuItem<'a>> {
    let disabled = PopupAttributeIdSet::new(metadata.attributes.get("disabled_options"));
    let checked = PopupAttributeIdSet::new(metadata.attributes.get("checked_options"));
    let focused = PopupAttributeIdSet::new(metadata.attributes.get("focused_options"));
    let hovered = PopupAttributeIdSet::new(metadata.attributes.get("hovered_options"));
    let pressed = PopupAttributeIdSet::new(metadata.attributes.get("pressed_options"));
    let loading = PopupAttributeIdSet::new(metadata.attributes.get("loading_options"));
    let focused_index = usize_attribute(metadata, "focused_index");
    let hovered_id = string_attribute(metadata, "hovered_option_id");

    let items_value = metadata
        .attributes
        .get("menu_items")
        .or_else(|| metadata.attributes.get("options"));
    items_value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    let mut item = menu_item(value)?;
                    item.apply_attribute_state(
                        index,
                        &disabled,
                        &checked,
                        &focused,
                        &hovered,
                        &pressed,
                        &loading,
                        focused_index,
                        hovered_id,
                    );
                    Some(item)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn menu_item<'a>(value: &'a Value) -> Option<RuntimePopupMenuItem<'a>> {
    match value {
        Value::String(raw) => Some(RuntimePopupMenuItem::from_raw(raw)),
        Value::Table(table) => RuntimePopupMenuItem::from_table(table),
        _ => None,
    }
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    match metadata.attributes.get(key)? {
        Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    }
}

#[derive(Clone)]
struct RuntimePopupMenuItem<'a> {
    id: &'a str,
    label: String,
    separator: bool,
    checked: bool,
    disabled: bool,
    hovered: bool,
    focused: bool,
    pressed: bool,
    loading: bool,
    danger: bool,
}

impl<'a> RuntimePopupMenuItem<'a> {
    fn from_raw(raw: &'a str) -> Self {
        if raw.trim() == "---" {
            return Self {
                id: "",
                label: String::new(),
                separator: true,
                checked: false,
                disabled: true,
                hovered: false,
                focused: false,
                pressed: false,
                loading: false,
                danger: false,
            };
        }

        let mut parts = raw.splitn(3, '|');
        let raw_label = parts.next().unwrap_or_default().trim();
        let flags = parts.next().unwrap_or_default();
        let id = flag_value(flags, "action").unwrap_or(raw_label);
        let label = flag_value(flags, "label")
            .or_else(|| flag_value(flags, "text"))
            .unwrap_or(raw_label)
            .to_string();
        Self {
            id,
            label,
            separator: false,
            checked: has_flag(flags, "checked"),
            disabled: has_flag(flags, "disabled"),
            hovered: has_flag(flags, "hovered"),
            focused: has_flag(flags, "focused"),
            pressed: has_flag(flags, "pressed"),
            loading: has_flag(flags, "loading"),
            danger: has_flag(flags, "danger"),
        }
    }

    fn from_table(table: &'a toml::map::Map<String, Value>) -> Option<Self> {
        let id = table
            .get("id")
            .or_else(|| table.get("value"))
            .or_else(|| table.get("label"))
            .or_else(|| table.get("text"))
            .and_then(Value::as_str)?
            .trim();
        let label = table
            .get("label")
            .or_else(|| table.get("text"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .unwrap_or(id)
            .to_string();
        Some(Self {
            id,
            label,
            separator: table_bool(table.get("separator")),
            checked: table_bool(table.get("checked")) || table_bool(table.get("selected")),
            disabled: table_bool(table.get("disabled")),
            hovered: table_bool(table.get("hovered")),
            focused: table_bool(table.get("focused")),
            pressed: table_bool(table.get("pressed")),
            loading: table_bool(table.get("loading")),
            danger: table_bool(table.get("danger")),
        })
    }

    fn apply_attribute_state(
        &mut self,
        row: usize,
        disabled: &PopupAttributeIdSet<'_>,
        checked: &PopupAttributeIdSet<'_>,
        focused: &PopupAttributeIdSet<'_>,
        hovered: &PopupAttributeIdSet<'_>,
        pressed: &PopupAttributeIdSet<'_>,
        loading: &PopupAttributeIdSet<'_>,
        focused_index: Option<usize>,
        hovered_id: Option<&str>,
    ) {
        if self.separator {
            return;
        }
        if self.matches_set(checked) {
            self.checked = true;
        }
        if self.matches_set(disabled) {
            self.disabled = true;
        }
        if self.matches_set(focused) || focused_index == Some(row) {
            self.focused = true;
        }
        if self.matches_set(hovered) || hovered_id.is_some_and(|value| self.matches_id(value)) {
            self.hovered = true;
        }
        if self.matches_set(pressed) {
            self.pressed = true;
        }
        if self.matches_set(loading) {
            self.loading = true;
        }
    }

    fn matches_set(&self, values: &PopupAttributeIdSet<'_>) -> bool {
        values.contains_any(self.id, &self.label)
    }

    fn matches_id(&self, value: &str) -> bool {
        !value.is_empty() && (self.id == value || self.label == value)
    }

    fn paint_state(&self) -> PopupRowPaintState {
        PopupRowPaintState::resolve(
            self.checked,
            self.hovered,
            self.focused,
            self.pressed,
            self.disabled,
            self.loading,
        )
    }

    fn text_color(&self, state: PopupRowPaintState) -> &'static str {
        state.text_color(self.danger)
    }
}

fn has_flag(flags: &str, expected: &str) -> bool {
    flags
        .split(',')
        .any(|flag| flag.trim().eq_ignore_ascii_case(expected))
}

fn flag_value<'a>(flags: &'a str, expected_key: &str) -> Option<&'a str> {
    flags.split(',').find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(expected_key)
            .then(|| value.trim())
            .filter(|value| !value.is_empty())
    })
}

fn table_bool(value: Option<&Value>) -> bool {
    value.and_then(Value::as_bool).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::RuntimePopupMenuItem;

    #[test]
    fn raw_item_keeps_explicit_action_id_separate_from_display_label() {
        let item = RuntimePopupMenuItem::from_raw(
            "Open Workspace|action=menu.item.open_project,icon=folder",
        );

        assert_eq!(item.id, "menu.item.open_project");
        assert_eq!(item.label, "Open Workspace");
    }

    #[test]
    fn raw_item_uses_label_as_legacy_id_fallback() {
        let item = RuntimePopupMenuItem::from_raw("Open Project|icon=folder");

        assert_eq!(item.id, "Open Project");
        assert_eq!(item.label, "Open Project");
    }
}
