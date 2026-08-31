use std::collections::HashSet;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentKeyboardAction, UiComponentState,
    UiValue, UiValueKind,
};

use super::{
    command_palette, notification_center, numeric, overlay, selection::option_is_disabled,
    tree_view,
};

mod menu;

pub(super) fn apply_keyboard_action(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) -> Result<(), UiComponentEventError> {
    if component_disabled(state, descriptor) {
        return Ok(());
    }
    if command_palette::apply_keyboard_action(state, descriptor, action)? {
        return Ok(());
    }
    if notification_center::apply_keyboard_action(state, descriptor, action)? {
        return Ok(());
    }

    match action {
        UiComponentKeyboardAction::Activate => activate(state, descriptor),
        UiComponentKeyboardAction::Cancel => cancel(state, descriptor),
        UiComponentKeyboardAction::BeginEdit => begin_edit(state, descriptor),
        UiComponentKeyboardAction::Next
        | UiComponentKeyboardAction::Previous
        | UiComponentKeyboardAction::First
        | UiComponentKeyboardAction::Last => navigate(state, descriptor, action),
        UiComponentKeyboardAction::Increment
        | UiComponentKeyboardAction::Decrement
        | UiComponentKeyboardAction::LargeIncrement
        | UiComponentKeyboardAction::LargeDecrement => numeric_step(state, descriptor, action),
    }
}

pub(super) fn apply_keyboard_text(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: &str,
) -> Result<(), UiComponentEventError> {
    if component_disabled(state, descriptor) {
        return Ok(());
    }
    if command_palette::apply_keyboard_text(state, descriptor, text)? {
        return Ok(());
    }
    menu::apply_keyboard_text(state, descriptor, text)
}

pub(super) fn apply_typeahead_expired(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    if component_disabled(state, descriptor) {
        return Ok(());
    }
    menu::apply_typeahead_expired(state, descriptor)
}

pub(super) fn sync_menu_search_filter(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    changed_property: &str,
) -> Result<(), UiComponentEventError> {
    menu::sync_search_filter(state, descriptor, changed_property)
}

fn activate(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    if menu::open_focused_submenu(state, descriptor) {
        return Ok(());
    }

    if is_popup_control(descriptor) {
        set_popup_open(state, descriptor, true)?;
    } else if descriptor.role == "radio" {
        set_checked(state, true);
        state.flags.selected = true;
        if let Some(option_id) = string_setting(state, descriptor, "option_id") {
            super::set_value(state, "group_value".to_string(), UiValue::String(option_id));
        }
    } else if toggle_group_is_multiple(state, descriptor) {
        toggle_multiple_toggle_group_focused_option(state, descriptor);
    } else if is_checkable_control(state, descriptor) {
        toggle_checked(state, descriptor);
    } else {
        super::set_value(state, "activated".to_string(), UiValue::Bool(true));
        state.flags.pressed = false;
    }
    Ok(())
}

fn cancel(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    if overlay::apply_cancel(state, descriptor)? {
        state.flags.pressed = false;
        state.flags.dragging = false;
        return Ok(());
    }

    if tree_view::apply_cancel_editing(state, descriptor)? {
        state.flags.pressed = false;
        state.flags.dragging = false;
        return Ok(());
    }

    if menu::close_active_submenu(state, descriptor) {
        state.flags.pressed = false;
        state.flags.dragging = false;
        return Ok(());
    }

    if is_popup_control(descriptor) || state.flags.popup_open || has_popup_open_value(state) {
        set_popup_open(state, descriptor, false)?;
    }
    state.flags.pressed = false;
    state.flags.dragging = false;
    Ok(())
}

fn begin_edit(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    tree_view::apply_begin_edit(state, descriptor)?;
    Ok(())
}

fn navigate(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) -> Result<(), UiComponentEventError> {
    let selection_control = is_indexed_keyboard_selection_control(descriptor);
    let focus_control = is_indexed_keyboard_focus_control(descriptor);
    if (!selection_control && !focus_control)
        || !bool_setting(state, descriptor, "keyboard_navigation", true)
    {
        return Ok(());
    }

    let options = indexed_keyboard_entries(state, descriptor);
    if options.is_empty() {
        return Ok(());
    }

    let eligibility = OptionEligibility::new(state, descriptor);
    let current = current_index(state, descriptor, &options);
    let next = match next_enabled_index(
        &eligibility,
        action,
        current,
        &options,
        focus_control && !bool_setting(state, descriptor, "disableListWrap", false),
        !focus_control || !bool_setting(state, descriptor, "disabledItemsFocusable", false),
    ) {
        Some(index) => index,
        None => return Ok(()),
    };

    state.flags.focused = true;
    super::set_value(state, "focused_index".to_string(), UiValue::Int(next));

    if selection_control && selection_follows_focus(state, descriptor) {
        set_indexed_selection(state, descriptor, next, &options);
    }
    Ok(())
}

fn numeric_step(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) -> Result<(), UiComponentEventError> {
    if tree_view::apply_keyboard_expand_collapse(state, descriptor, action)? {
        return Ok(());
    }

    if !descriptor
        .prop("value")
        .is_some_and(|schema| matches!(schema.value_kind, UiValueKind::Float | UiValueKind::Int))
    {
        return Ok(());
    }

    let (delta, step_property) = match action {
        UiComponentKeyboardAction::Increment => (1.0, "step"),
        UiComponentKeyboardAction::Decrement => (-1.0, "step"),
        UiComponentKeyboardAction::LargeIncrement => (1.0, "large_step"),
        UiComponentKeyboardAction::LargeDecrement => (-1.0, "large_step"),
        _ => return Ok(()),
    };
    numeric::apply_numeric_drag(
        state,
        descriptor,
        numeric_keyboard_property(state, descriptor),
        delta,
        step_property,
    )
}

fn numeric_keyboard_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> String {
    if descriptor.role == "range-slider" && descriptor.prop("range_min").is_some() {
        return range_slider_keyboard_property(state, descriptor).to_string();
    }
    "value".to_string()
}

fn range_slider_keyboard_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    for property in ["active_thumb", "focused_thumb", "keyboard_thumb", "thumb"] {
        let Some(thumb) = string_setting(state, descriptor, property) else {
            continue;
        };
        match thumb.trim().to_ascii_lowercase().as_str() {
            "lower" | "min" | "minimum" | "start" | "range_min" => return "range_min",
            "upper" | "max" | "maximum" | "end" | "value" => return "value",
            _ => {}
        }
    }

    for property in ["active_thumb_index", "focused_thumb_index", "thumb_index"] {
        let Some(index) = int_setting(state, descriptor, property) else {
            continue;
        };
        return if index <= 0 { "range_min" } else { "value" };
    }

    "value"
}

fn toggle_checked(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    let next = if descriptor.role == "checkbox"
        && bool_setting(state, descriptor, "indeterminate", false)
    {
        let resolved = bool_setting(state, descriptor, "indeterminate_resolves_to_checked", true);
        super::set_value(state, "indeterminate".to_string(), UiValue::Bool(false));
        resolved
    } else {
        !current_checked(state, descriptor)
    };
    set_checked(state, next);
}

fn toggle_multiple_toggle_group_focused_option(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) {
    let options = option_ids(state, descriptor);
    if options.is_empty() {
        return;
    }

    let current = current_index(state, descriptor, &options).clamp(0, (options.len() - 1) as i64);
    let Some(option_id) = options.get(current as usize) else {
        return;
    };
    let eligibility = OptionEligibility::new(state, descriptor);
    if eligibility.is_disabled(option_id) {
        return;
    }

    let next_selected = !state
        .values
        .get("value")
        .is_some_and(|value| option_id_list_contains(value, option_id));
    set_multiple_toggle_group_value(state, option_id, next_selected);
    state.flags.focused = true;
    state.flags.selected = next_selected;
}

fn set_multiple_toggle_group_value(state: &mut UiComponentState, option_id: &str, selected: bool) {
    let mut values = match state.values.remove("value") {
        Some(value) => multi_selection_values(value),
        None => Vec::new(),
    };

    if selected {
        if !values
            .iter()
            .any(|value| option_id_list_contains(value, option_id))
        {
            values.push(UiValue::Enum(option_id.to_string()));
        }
    } else {
        values.retain(|value| !option_id_list_contains(value, option_id));
    }

    super::set_value(state, "value".to_string(), UiValue::Array(values));
}

fn multi_selection_values(value: UiValue) -> Vec<UiValue> {
    match value {
        UiValue::Array(values) => values
            .into_iter()
            .filter_map(multi_selection_value_entry)
            .collect(),
        UiValue::Flags(values) => values.into_iter().map(UiValue::Enum).collect(),
        value => multi_selection_value_entry(value).into_iter().collect(),
    }
}

fn multi_selection_value_entry(value: UiValue) -> Option<UiValue> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => {
            Some(UiValue::Enum(value))
        }
        UiValue::Null => None,
        value => Some(value),
    }
}

fn set_checked(state: &mut UiComponentState, checked: bool) {
    state.flags.checked = checked;
    super::set_value(state, "checked".to_string(), UiValue::Bool(checked));
}

fn set_popup_open(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    open: bool,
) -> Result<(), UiComponentEventError> {
    if open {
        overlay::open_popup(state, descriptor)?;
    } else {
        overlay::close_popup(state, descriptor)?;
    }
    Ok(())
}

fn current_index(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    options: &[String],
) -> i64 {
    int_setting(state, descriptor, "focused_index")
        .or_else(|| int_setting(state, descriptor, "selected_index"))
        .or_else(|| current_value_index(state, options))
        .unwrap_or(0)
}

fn current_value_index(state: &UiComponentState, options: &[String]) -> Option<i64> {
    ["value", "value_text", "group_value"]
        .into_iter()
        .filter_map(|property| state.values.get(property).and_then(string_value))
        .find_map(|value| {
            options
                .iter()
                .position(|option| option == &value)
                .map(|index| index as i64)
        })
}

fn option_ids(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> Vec<String> {
    option_entries(state, descriptor)
        .into_iter()
        .map(|option| option.id)
        .collect()
}

fn indexed_keyboard_entries(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Vec<String> {
    indexed_entry_property_candidates(descriptor)
        .iter()
        .copied()
        .filter_map(|property| state.values.get(property))
        .flat_map(option_entry_list)
        .map(|option| option.id)
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|option| !option.is_empty())
        .collect()
}

fn indexed_entry_property_candidates(
    descriptor: &UiComponentDescriptor,
) -> &'static [&'static str] {
    if tree_view::is_tree_view(descriptor) {
        &["nodes", "items", "options"]
    } else if is_row_keyboard_collection(descriptor) {
        &["rows", "items", "options"]
    } else {
        &["options"]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OptionEntry {
    id: String,
    text: String,
}

fn option_entries(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Vec<OptionEntry> {
    state
        .values
        .get("options")
        .or_else(|| {
            descriptor
                .prop("options")
                .and_then(|schema| schema.default_value.as_ref())
        })
        .map(option_entry_list)
        .unwrap_or_default()
}

fn option_entry_list(value: &UiValue) -> Vec<OptionEntry> {
    match value {
        UiValue::Array(values) => values.iter().flat_map(option_entry_list).collect(),
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => {
            vec![OptionEntry {
                id: value.clone(),
                text: value.clone(),
            }]
        }
        UiValue::Map(values) => {
            let ids = values
                .get("id")
                .or_else(|| values.get("value"))
                .or_else(|| values.get("row_id"))
                .or_else(|| values.get("rowId"))
                .or_else(|| values.get("node_id"))
                .or_else(|| values.get("nodeId"))
                .or_else(|| values.get("key"))
                .map(option_id_list)
                .unwrap_or_default();
            let text = option_label_text(values);
            ids.into_iter()
                .filter(|id| !id.is_empty())
                .map(|id| OptionEntry {
                    text: text.clone().unwrap_or_else(|| id.clone()),
                    id,
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn option_label_text(values: &std::collections::BTreeMap<String, UiValue>) -> Option<String> {
    ["label", "text", "title", "value_text", "value", "id"]
        .into_iter()
        .filter_map(|property| values.get(property).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn next_enabled_index(
    eligibility: &OptionEligibility<'_>,
    action: UiComponentKeyboardAction,
    current: i64,
    options: &[String],
    wrap: bool,
    skip_disabled: bool,
) -> Option<i64> {
    let max_index = (options.len() - 1) as i64;
    let focusable =
        |index: i64| !skip_disabled || !eligibility.is_disabled(&options[index as usize]);
    if !(0..=max_index).contains(&current) {
        return match action {
            UiComponentKeyboardAction::Previous | UiComponentKeyboardAction::Last => {
                (0..=max_index).rev().find(|index| focusable(*index))
            }
            _ => (0..=max_index).find(|index| focusable(*index)),
        };
    }

    match action {
        UiComponentKeyboardAction::First => (0..=max_index).find(|index| focusable(*index)),
        UiComponentKeyboardAction::Last => (0..=max_index).rev().find(|index| focusable(*index)),
        UiComponentKeyboardAction::Previous => (0..current)
            .rev()
            .find(|index| focusable(*index))
            .or_else(|| {
                wrap.then(|| {
                    ((current + 1)..=max_index)
                        .rev()
                        .find(|index| focusable(*index))
                })
                .flatten()
            })
            .or_else(|| focusable(current).then_some(current)),
        UiComponentKeyboardAction::Next => ((current + 1)..=max_index)
            .find(|index| focusable(*index))
            .or_else(|| {
                wrap.then(|| (0..current).find(|index| focusable(*index)))
                    .flatten()
            })
            .or_else(|| focusable(current).then_some(current)),
        _ => focusable(current).then_some(current),
    }
}

fn option_id_list(value: &UiValue) -> Vec<String> {
    match value {
        UiValue::Array(values) => values
            .iter()
            .flat_map(option_id_list)
            .filter(|value| !value.is_empty())
            .collect(),
        UiValue::String(value) | UiValue::Enum(value) => vec![value.clone()],
        UiValue::Map(values) => values
            .get("id")
            .or_else(|| values.get("value"))
            .map(option_id_list)
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

struct ExplicitOptionEligibility<'a> {
    disabled: HashSet<&'a str>,
}

impl<'a> ExplicitOptionEligibility<'a> {
    fn new(state: &'a UiComponentState, descriptor: &'a UiComponentDescriptor) -> Self {
        let mut disabled = descriptor
            .prop("options")
            .into_iter()
            .flat_map(|schema| schema.options.iter())
            .filter(|option| option.disabled)
            .map(|option| option.id.as_str())
            .collect::<HashSet<_>>();
        if let Some(value) = state.values.get("disabled_options") {
            collect_option_id_refs(value, &mut disabled);
        }
        Self { disabled }
    }

    fn is_disabled(&self, option_id: &str) -> bool {
        self.disabled.contains(option_id)
    }
}

struct OptionEligibility<'a> {
    explicit: ExplicitOptionEligibility<'a>,
    search_filter: Option<menu::MenuSearchFilter<'a>>,
}

impl<'a> OptionEligibility<'a> {
    fn new(state: &'a UiComponentState, descriptor: &'a UiComponentDescriptor) -> Self {
        Self {
            explicit: ExplicitOptionEligibility::new(state, descriptor),
            search_filter: menu::option_search_filter(state, descriptor),
        }
    }

    fn is_disabled(&self, option_id: &str) -> bool {
        self.explicit.is_disabled(option_id)
            || self
                .search_filter
                .as_ref()
                .is_some_and(|filter| menu::option_is_hidden_by_search_filter(filter, option_id))
    }
}

fn collect_option_id_refs<'a>(value: &'a UiValue, ids: &mut HashSet<&'a str>) {
    match value {
        UiValue::Array(values) => {
            for value in values {
                collect_option_id_refs(value, ids);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => {
            ids.insert(value.as_str());
        }
        UiValue::Flags(values) => {
            ids.extend(values.iter().map(String::as_str));
        }
        _ => {}
    }
}

fn option_id_list_contains(value: &UiValue, option_id: &str) -> bool {
    match value {
        UiValue::Array(values) => values
            .iter()
            .any(|value| option_id_list_contains(value, option_id)),
        UiValue::String(value) | UiValue::Enum(value) => value == option_id,
        UiValue::Flags(values) => values.iter().any(|value| value == option_id),
        _ => false,
    }
}

fn current_checked(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    state
        .values
        .get("checked")
        .and_then(bool_value)
        .or_else(|| {
            descriptor
                .prop("checked")
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(bool_value)
        })
        .unwrap_or(state.flags.checked)
}

fn component_disabled(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    state.flags.disabled || bool_setting(state, descriptor, "disabled", false)
}

fn is_checkable_control(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "checkbox" | "switch" | "toggle-button"
    ) || (descriptor.role == "icon-button" && bool_setting(state, descriptor, "checkable", false))
}

fn is_popup_control(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "select" | "autocomplete" | "dropdown" | "menu" | "menubar"
    )
}

fn is_tabs(descriptor: &UiComponentDescriptor) -> bool {
    descriptor.role == "tabs" || descriptor.id == "Tabs"
}

fn is_radio_group(descriptor: &UiComponentDescriptor) -> bool {
    descriptor.role == "radio-group" || descriptor.id == "RadioGroup"
}

fn is_toggle_button_group(descriptor: &UiComponentDescriptor) -> bool {
    descriptor.role == "toggle-button-group" || descriptor.id == "ToggleButtonGroup"
}

fn is_row_keyboard_collection(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "list-view" | "asset-list" | "asset-grid" | "data-grid" | "mui-x-data-grid" | "table"
    ) || matches!(
        descriptor.id.as_str(),
        "ListView" | "AssetList" | "AssetGrid" | "DataGrid" | "Table"
    )
}

fn is_indexed_keyboard_selection_control(descriptor: &UiComponentDescriptor) -> bool {
    is_tabs(descriptor)
        || is_radio_group(descriptor)
        || is_toggle_button_group(descriptor)
        || tree_view::is_tree_view(descriptor)
        || is_row_keyboard_collection(descriptor)
}

fn is_indexed_keyboard_focus_control(descriptor: &UiComponentDescriptor) -> bool {
    menu::is_focus_control(descriptor)
}

fn selection_follows_focus(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    if is_radio_group(descriptor) {
        return bool_setting(state, descriptor, "selection_follows_focus", true);
    }
    if is_toggle_button_group(descriptor) && toggle_group_is_multiple(state, descriptor) {
        return false;
    }
    bool_setting(state, descriptor, "selection_follows_focus", false)
        || bool_setting(state, descriptor, "selectionFollowsFocus", false)
}

fn toggle_group_is_multiple(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    is_toggle_button_group(descriptor)
        && string_setting(state, descriptor, "selection_state")
            .is_some_and(|value| value == "multiple")
}

fn set_indexed_selection(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    index: i64,
    options: &[String],
) {
    super::set_value(state, "selected_index".to_string(), UiValue::Int(index));
    state.flags.selected = true;

    if let Some(option_id) = options.get(index as usize) {
        super::set_value(
            state,
            "value".to_string(),
            UiValue::String(option_id.clone()),
        );
        if is_radio_group(descriptor) || state.values.contains_key("group_value") {
            super::set_value(
                state,
                "group_value".to_string(),
                UiValue::String(option_id.clone()),
            );
        }
        if descriptor.prop("value_text").is_some() || state.values.contains_key("value_text") {
            super::set_value(
                state,
                "value_text".to_string(),
                UiValue::String(option_id.clone()),
            );
        }
    }
}

fn has_popup_open_value(state: &UiComponentState) -> bool {
    ["popup_open", "popupOpen", "open"]
        .into_iter()
        .any(|property| state.values.contains_key(property))
}

fn bool_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    default_value: bool,
) -> bool {
    state
        .values
        .get(property)
        .and_then(bool_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(bool_value)
        })
        .unwrap_or(default_value)
}

fn bool_value(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn int_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<i64> {
    state.values.get(property).and_then(int_value).or_else(|| {
        descriptor
            .prop(property)
            .and_then(|schema| schema.default_value.as_ref())
            .and_then(int_value)
    })
}

fn int_value(value: &UiValue) -> Option<i64> {
    match value {
        UiValue::Int(value) => Some(*value),
        UiValue::Float(value) => Some(value.round() as i64),
        _ => None,
    }
}

fn string_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    state
        .values
        .get(property)
        .and_then(string_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(string_value)
        })
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}
