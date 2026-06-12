use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::popup_rows::{
    menu_row_height, popup_base_z, push_popup_background, push_popup_row_label,
    push_popup_row_surface, push_popup_separator, PopupRowPaintState,
};

pub(super) fn popup_menu_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    frame: UiFrame,
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
    let items = menu_items(metadata);
    if items.is_empty() {
        return Vec::new();
    }

    let base_z = popup_base_z(z_index);
    let Some(row_height) = menu_row_height(frame, items.len()) else {
        return Vec::new();
    };
    let mut commands = Vec::new();
    push_popup_background(&mut commands, node_id, frame, clip_frame, base_z, opacity);

    for (row, item) in items.iter().enumerate() {
        let row_frame = UiFrame::new(
            frame.x,
            frame.y + row as f32 * row_height,
            frame.width.max(1.0),
            row_height,
        );
        let row_z = base_z.saturating_add(1 + row as i32);
        if item.separator {
            push_popup_separator(
                &mut commands,
                node_id,
                row_frame,
                clip_frame,
                row_z,
                opacity,
            );
            continue;
        }
        let row_state = item.paint_state();
        push_popup_row_surface(
            &mut commands,
            node_id,
            row_frame,
            clip_frame,
            row_z,
            row_state,
            opacity,
        );
        push_popup_row_label(
            &mut commands,
            node_id,
            row_frame,
            clip_frame,
            row_z.saturating_add(2),
            item.label.clone(),
            item.text_color(row_state),
            row_state,
            opacity,
        );
    }

    commands
}

fn is_open_context_menu(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "ContextActionMenu"
        && bool_attribute(metadata, "popup_open").or_else(|| bool_attribute(metadata, "open"))
            == Some(true)
}

fn menu_items(metadata: &UiTemplateNodeMetadata) -> Vec<RuntimePopupMenuItem> {
    metadata
        .attributes
        .get("menu_items")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(menu_item).collect())
        .unwrap_or_default()
}

fn menu_item(value: &Value) -> Option<RuntimePopupMenuItem> {
    match value {
        Value::String(raw) => Some(RuntimePopupMenuItem::from_raw(raw)),
        Value::Table(table) => table
            .get("label")
            .or_else(|| table.get("text"))
            .or_else(|| table.get("value"))
            .and_then(Value::as_str)
            .map(RuntimePopupMenuItem::from_raw),
        _ => None,
    }
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

#[derive(Clone)]
struct RuntimePopupMenuItem {
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

impl RuntimePopupMenuItem {
    fn from_raw(raw: &str) -> Self {
        if raw.trim() == "---" {
            return Self {
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
        let label = parts.next().unwrap_or_default().trim().to_string();
        let flags = parts.next().unwrap_or_default();
        Self {
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
