use std::collections::BTreeSet;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use super::popup_rows::{
    option_popup_frame_within, option_row_frame_within, popup_base_z, push_popup_background,
    push_popup_row_label, push_popup_row_surface, PopupRowPaintState,
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
    let popup_frame = option_popup_frame_within(frame, options.len(), layout_bounds);
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
        let Some(row_frame) = option_row_frame_within(frame, options.len(), row, layout_bounds)
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

fn option_popup_layout_bounds(
    control_frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> Option<UiFrame> {
    clip_frame.filter(|clip_frame| *clip_frame != control_frame)
}

fn is_open_option_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "ComboBox" | "Dropdown" | "Select"
    ) && bool_attribute(metadata, "popup_open").or_else(|| bool_attribute(metadata, "open"))
        == Some(true)
}

fn option_rows(metadata: &UiTemplateNodeMetadata) -> Vec<RuntimePopupOption> {
    let selected = selected_option_ids(metadata.attributes.get("value"));
    let disabled = option_id_set(metadata.attributes.get("disabled_options"));
    let special = option_id_set(metadata.attributes.get("special_options"));
    let focused = option_id_set(metadata.attributes.get("focused_options"));
    let hovered = option_id_set(metadata.attributes.get("hovered_options"));
    let pressed = option_id_set(metadata.attributes.get("pressed_options"));
    let loading = option_id_set(metadata.attributes.get("loading_options"));

    metadata
        .attributes
        .get("options")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    option_row(
                        value, &selected, &disabled, &special, &focused, &hovered, &pressed,
                        &loading,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn option_row(
    value: &Value,
    selected: &BTreeSet<String>,
    disabled: &BTreeSet<String>,
    special: &BTreeSet<String>,
    focused: &BTreeSet<String>,
    hovered: &BTreeSet<String>,
    pressed: &BTreeSet<String>,
    loading: &BTreeSet<String>,
) -> Option<RuntimePopupOption> {
    let mut option = RuntimePopupOption::from_value(value)?;
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
    if hovered.contains(&option.id) || hovered.contains(&option.label) {
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

fn selected_option_ids(value: Option<&Value>) -> BTreeSet<String> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimePopupOption {
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
