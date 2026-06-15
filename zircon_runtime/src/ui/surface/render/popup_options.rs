use std::collections::BTreeSet;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use super::popup_rows::{
    option_popup_frame_within, option_popup_layout_bounds, option_row_frame_within, popup_base_z,
    push_popup_background, push_popup_row_label, push_popup_row_surface, PopupRowPaintState,
};

pub(super) fn popup_option_render_commands(
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
    if !is_open_option_component(metadata) {
        return Vec::new();
    }
    let options = option_rows(metadata);
    if options.is_empty() {
        return Vec::new();
    }

    let layout_bounds = option_popup_layout_bounds(frame, clip_frame);
    let popup_frame = option_popup_frame_within(metadata, frame, options.len(), layout_bounds);
    let Some(popup_frame) = popup_frame else {
        return Vec::new();
    };
    let render_clip = layout_bounds;
    let base_z = popup_base_z(z_index);
    let mut commands = Vec::new();
    push_popup_background(
        &mut commands,
        node_id,
        popup_frame,
        render_clip,
        base_z,
        opacity,
    );

    for (row, option) in options.iter().enumerate() {
        let Some(row_frame) =
            option_row_frame_within(metadata, frame, options.len(), row, layout_bounds)
        else {
            continue;
        };
        let row_z = base_z.saturating_add(1 + row as i32);
        let row_state = option.paint_state();
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
            option.label.clone(),
            row_state.text_color(false),
            row_state,
            opacity,
        );
    }

    commands
}

fn is_open_option_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "ComboBox" | "Dropdown" | "Select" | "DropdownPopup"
    ) && bool_attribute(metadata, "popup_open").or_else(|| bool_attribute(metadata, "open"))
        == Some(true)
}

fn option_rows(metadata: &UiTemplateNodeMetadata) -> Vec<RuntimePopupOption> {
    let selected = selected_option_ids(metadata);
    let disabled = option_id_set(metadata.attributes.get("disabled_options"));
    let special = option_id_set(metadata.attributes.get("special_options"));
    let focused = option_id_set(metadata.attributes.get("focused_options"));
    let hovered = option_id_set(metadata.attributes.get("hovered_options"));
    let pressed = option_id_set(metadata.attributes.get("pressed_options"));
    let loading = option_id_set(metadata.attributes.get("loading_options"));
    let focused_index = usize_attribute(metadata, "focused_index");
    let hovered_id = string_attribute(metadata, "hovered_option_id");

    metadata
        .attributes
        .get("options")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    option_row(
                        value,
                        index,
                        &selected,
                        &disabled,
                        &special,
                        &focused,
                        &hovered,
                        &pressed,
                        &loading,
                        focused_index,
                        hovered_id,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn option_row(
    value: &Value,
    index: usize,
    selected: &BTreeSet<String>,
    disabled: &BTreeSet<String>,
    special: &BTreeSet<String>,
    focused: &BTreeSet<String>,
    hovered: &BTreeSet<String>,
    pressed: &BTreeSet<String>,
    loading: &BTreeSet<String>,
    focused_index: Option<usize>,
    hovered_id: Option<&str>,
) -> Option<RuntimePopupOption> {
    let mut option = RuntimePopupOption::from_value(value)?;
    option.index = index;
    if selected.contains(&option.id) || selected.contains(&option.label) {
        option.selected = true;
    }
    if disabled.contains(&option.id) || disabled.contains(&option.label) {
        option.disabled = true;
    }
    if special.contains(&option.id) || special.contains(&option.label) {
        option.special = true;
    }
    if focused.contains(&option.id) || focused.contains(&option.label) {
        option.focused = true;
    }
    if focused_index == Some(option.index) {
        option.focused = true;
    }
    if hovered.contains(&option.id) || hovered.contains(&option.label) {
        option.hovered = true;
    }
    if hovered_id.is_some_and(|value| option.matches_id(value)) {
        option.hovered = true;
    }
    if pressed.contains(&option.id) || pressed.contains(&option.label) {
        option.pressed = true;
    }
    if loading.contains(&option.id) || loading.contains(&option.label) {
        option.loading = true;
    }
    Some(option)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn option_id_set(value: Option<&Value>) -> BTreeSet<String> {
    option_values(value).into_iter().collect()
}

fn selected_option_ids(metadata: &UiTemplateNodeMetadata) -> BTreeSet<String> {
    let mut selected = value_selected_option_ids(metadata.attributes.get("value"));
    selected.extend(option_values(metadata.attributes.get("selected_options")));
    selected.extend(option_values(metadata.attributes.get("selectedOptions")));
    selected
}

fn value_selected_option_ids(value: Option<&Value>) -> BTreeSet<String> {
    let Some(value) = value else {
        return BTreeSet::new();
    };

    match UiValue::from_toml(value) {
        UiValue::String(value) | UiValue::Enum(value) => BTreeSet::from([value]),
        UiValue::Flags(values) => values.into_iter().collect(),
        UiValue::Array(values) => values
            .into_iter()
            .map(|value| value.display_text())
            .filter(|value| !value.is_empty())
            .collect(),
        value => {
            let text = value.display_text();
            if text.is_empty() {
                BTreeSet::new()
            } else {
                BTreeSet::from([text])
            }
        }
    }
}

fn option_values(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::Array(values) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        Value::String(value) => vec![value.clone()],
        _ => Vec::new(),
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimePopupOption {
    index: usize,
    id: String,
    label: String,
    selected: bool,
    disabled: bool,
    special: bool,
    focused: bool,
    hovered: bool,
    pressed: bool,
    loading: bool,
}

impl RuntimePopupOption {
    fn paint_state(&self) -> PopupRowPaintState {
        PopupRowPaintState::resolve(
            self.selected || self.special,
            self.hovered,
            self.focused,
            self.pressed,
            self.disabled,
            self.loading,
        )
    }

    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::String(raw) => Some(Self::from_raw(raw)),
            Value::Table(table) => {
                let id = table
                    .get("id")
                    .or_else(|| table.get("value"))
                    .or_else(|| table.get("label"))
                    .or_else(|| table.get("text"))
                    .and_then(Value::as_str)?
                    .trim()
                    .to_string();
                let label = table
                    .get("label")
                    .or_else(|| table.get("text"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|label| !label.is_empty())
                    .unwrap_or(id.as_str())
                    .to_string();
                Some(Self {
                    index: 0,
                    id,
                    label,
                    selected: table_bool(table.get("selected")) || table_bool(table.get("checked")),
                    disabled: table_bool(table.get("disabled")),
                    special: table_bool(table.get("special")),
                    focused: table_bool(table.get("focused")),
                    hovered: table_bool(table.get("hovered")),
                    pressed: table_bool(table.get("pressed")),
                    loading: table_bool(table.get("loading")),
                })
            }
            _ => None,
        }
    }

    fn from_raw(raw: &str) -> Self {
        let mut parts = raw.splitn(3, '|');
        let id = parts.next().unwrap_or_default().trim().to_string();
        let flags = parts.next().unwrap_or_default();
        let label = flag_value(flags, "label")
            .or_else(|| flag_value(flags, "text"))
            .unwrap_or_else(|| id.clone());
        Self {
            index: 0,
            id,
            label,
            selected: has_flag(flags, "selected") || has_flag(flags, "checked"),
            disabled: has_flag(flags, "disabled"),
            special: has_flag(flags, "special"),
            focused: has_flag(flags, "focused"),
            hovered: has_flag(flags, "hovered"),
            pressed: has_flag(flags, "pressed"),
            loading: has_flag(flags, "loading"),
        }
    }

    fn matches_id(&self, value: &str) -> bool {
        !value.is_empty() && (self.id == value || self.label == value)
    }
}

fn table_bool(value: Option<&Value>) -> bool {
    value.and_then(Value::as_bool).unwrap_or(false)
}

fn has_flag(flags: &str, expected: &str) -> bool {
    flags
        .split(',')
        .any(|flag| flag.trim().eq_ignore_ascii_case(expected))
}

fn flag_value(flags: &str, expected_key: &str) -> Option<String> {
    flags.split(',').find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(expected_key)
            .then(|| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}
