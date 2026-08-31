use std::collections::BTreeSet;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use super::popup_rows::{
    PopupAttributeIdSet, PopupRowPaintState, option_popup_frame_within, option_popup_layout_bounds,
    popup_base_z, popup_row_frame, push_popup_background, push_popup_row_label,
    push_popup_row_surface,
};

pub(super) fn popup_option_render_commands(
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
    if !is_open_option_component(metadata) {
        return Vec::new();
    }
    let Some(anchor_frame) = anchor_frame else {
        return Vec::new();
    };
    let options = option_rows(metadata);
    if options.is_empty() {
        return Vec::new();
    }
    let option_count = options.len();

    let layout_bounds = option_popup_layout_bounds(frame, clip_frame);
    let popup_frame =
        option_popup_frame_within(metadata, frame, anchor_frame, option_count, layout_bounds);
    let Some(popup_frame) = popup_frame else {
        return Vec::new();
    };
    let render_clip = layout_bounds;
    let base_z = popup_base_z(z_index);
    let mut commands = Vec::with_capacity(option_count.saturating_mul(3).saturating_add(3));
    push_popup_background(
        &mut commands,
        node_id,
        metadata,
        popup_frame,
        render_clip,
        base_z,
        opacity,
    );

    for (row, option) in options.into_iter().enumerate() {
        let Some(row_frame) = popup_row_frame(metadata, popup_frame, option_count, row) else {
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
            option.label,
            row_state.text_color(false),
            row_state,
            opacity,
        );
    }

    commands
}

pub(super) fn popup_option_may_emit_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    is_open_option_component(metadata)
        && metadata
            .attributes
            .get("options")
            .and_then(Value::as_array)
            .is_some_and(|options| !options.is_empty())
}

fn is_open_option_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "ComboBox" | "Dropdown" | "Select" | "DropdownPopup"
    ) && bool_attribute(metadata, "popup_open").or_else(|| bool_attribute(metadata, "open"))
        == Some(true)
}

fn option_rows<'a>(metadata: &'a UiTemplateNodeMetadata) -> Vec<RuntimePopupOption<'a>> {
    let selected = value_selected_option_ids(metadata.attributes.get("value"));
    let selected_options = PopupAttributeIdSet::new(metadata.attributes.get("selected_options"));
    let selected_options_camel =
        PopupAttributeIdSet::new(metadata.attributes.get("selectedOptions"));
    let disabled = PopupAttributeIdSet::new(metadata.attributes.get("disabled_options"));
    let special = PopupAttributeIdSet::new(metadata.attributes.get("special_options"));
    let focused = PopupAttributeIdSet::new(metadata.attributes.get("focused_options"));
    let hovered = PopupAttributeIdSet::new(metadata.attributes.get("hovered_options"));
    let pressed = PopupAttributeIdSet::new(metadata.attributes.get("pressed_options"));
    let loading = PopupAttributeIdSet::new(metadata.attributes.get("loading_options"));
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
                        &selected_options,
                        &selected_options_camel,
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

fn option_row<'a>(
    value: &'a Value,
    index: usize,
    selected: &BTreeSet<String>,
    selected_options: &PopupAttributeIdSet<'_>,
    selected_options_camel: &PopupAttributeIdSet<'_>,
    disabled: &PopupAttributeIdSet<'_>,
    special: &PopupAttributeIdSet<'_>,
    focused: &PopupAttributeIdSet<'_>,
    hovered: &PopupAttributeIdSet<'_>,
    pressed: &PopupAttributeIdSet<'_>,
    loading: &PopupAttributeIdSet<'_>,
    focused_index: Option<usize>,
    hovered_id: Option<&str>,
) -> Option<RuntimePopupOption<'a>> {
    let mut option = RuntimePopupOption::from_value(value)?;
    option.index = index;
    if selected.contains(option.id)
        || selected.contains(&option.label)
        || selected_options.contains_any(option.id, &option.label)
        || selected_options_camel.contains_any(option.id, &option.label)
    {
        option.selected = true;
    }
    if disabled.contains_any(option.id, &option.label) {
        option.disabled = true;
    }
    if special.contains_any(option.id, &option.label) {
        option.special = true;
    }
    if focused.contains_any(option.id, &option.label) {
        option.focused = true;
    }
    if focused_index == Some(option.index) {
        option.focused = true;
    }
    if hovered.contains_any(option.id, &option.label) {
        option.hovered = true;
    }
    if hovered_id.is_some_and(|value| option.matches_id(value)) {
        option.hovered = true;
    }
    if pressed.contains_any(option.id, &option.label) {
        option.pressed = true;
    }
    if loading.contains_any(option.id, &option.label) {
        option.loading = true;
    }
    Some(option)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
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
struct RuntimePopupOption<'a> {
    index: usize,
    id: &'a str,
    label: String,
    selected: bool,
    disabled: bool,
    special: bool,
    focused: bool,
    hovered: bool,
    pressed: bool,
    loading: bool,
}

impl<'a> RuntimePopupOption<'a> {
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

    fn from_value(value: &'a Value) -> Option<Self> {
        match value {
            Value::String(raw) => Some(Self::from_raw(raw)),
            Value::Table(table) => {
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

    fn from_raw(raw: &'a str) -> Self {
        let mut parts = raw.splitn(3, '|');
        let id = parts.next().unwrap_or_default().trim();
        let flags = parts.next().unwrap_or_default();
        let label = flag_value(flags, "label")
            .or_else(|| flag_value(flags, "text"))
            .unwrap_or(id)
            .to_string();
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

fn flag_value<'a>(flags: &'a str, expected_key: &str) -> Option<&'a str> {
    flags.split(',').find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(expected_key)
            .then(|| value.trim())
            .filter(|value| !value.is_empty())
    })
}
