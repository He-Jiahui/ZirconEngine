use std::collections::HashSet;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValue,
};

use super::super::text_search::{contains_lowercase_query, starts_with_lowercase_query};

mod submenu;

#[cfg(test)]
#[path = "menu/tests/search_capacity_tests.rs"]
mod search_capacity_tests;

#[cfg(test)]
#[path = "menu/tests/search_streaming_tests.rs"]
mod search_streaming_tests;

#[cfg(test)]
#[path = "menu/tests/typeahead_option_id_tests.rs"]
mod typeahead_option_id_tests;

#[cfg(test)]
#[path = "menu/tests/typeahead_text_normalization_tests.rs"]
mod typeahead_text_normalization_tests;

#[cfg(test)]
#[path = "menu/tests/typeahead_append_reuse_tests.rs"]
mod typeahead_append_reuse_tests;

#[cfg(test)]
#[path = "menu/tests/typeahead_search_projection_tests.rs"]
mod typeahead_search_projection_tests;

#[cfg(test)]
#[path = "menu/tests/search_query_borrow_tests.rs"]
mod search_query_borrow_tests;

#[cfg(test)]
#[path = "menu/tests/search_filter_accumulator_tests.rs"]
mod search_filter_accumulator_tests;

#[cfg(test)]
#[path = "menu/tests/label_borrow_tests.rs"]
mod label_borrow_tests;

#[cfg(test)]
#[path = "menu/tests/child_values_iterator_tests.rs"]
mod child_values_iterator_tests;

pub(super) use submenu::{close_active_submenu, open_focused_submenu};

const MENU_TYPEAHEAD_BUFFER: &str = "typeahead_buffer";
const MENU_TYPEAHEAD_BUFFER_EXPIRED: &str = "typeahead_buffer_expired";
const MENU_ALLOW_SEARCH: &str = "allow_search";
const MENU_ALLOW_SEARCH_CAMEL: &str = "allowSearch";
const MENU_FILTERED_OPTION_IDS: &str = "filtered_option_ids";
const MENU_FILTER_NO_RESULTS: &str = "filter_no_results";
const MENU_SEARCH_BAR_ENABLED_ON_ITEM_COUNT: &str = "search_bar_enabled_on_item_count";
const MENU_SEARCH_BAR_ENABLED_ON_ITEM_COUNT_CAMEL: &str = "searchBarEnabledOnItemCount";
const MENU_SEARCH_QUERY: &str = "search_query";
const MENU_CHILD_PROPERTY_NAMES: [&str; 6] = [
    "children", "items", "submenu", "sub_menu", "subMenu", "options",
];

pub(super) fn apply_keyboard_text(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: &str,
) -> Result<(), UiComponentEventError> {
    if !is_focus_control(descriptor)
        || !super::bool_setting(state, descriptor, "keyboard_navigation", true)
    {
        return Ok(());
    }

    let Some(searches) = menu_typeahead_searches(state, descriptor, text) else {
        return Ok(());
    };
    let options = super::option_entries(state, descriptor);
    if options.is_empty() {
        return Ok(());
    }

    let current = super::current_option_entry_index(state, descriptor, &options);
    let matched = {
        let eligibility = super::OptionEligibility::new(state, descriptor);
        searches.iter().find_map(|search| {
            next_text_match_index(
                &eligibility,
                current,
                &options,
                &search.buffer,
                !super::bool_setting(state, descriptor, "disableListWrap", false),
                !super::bool_setting(state, descriptor, "disabledItemsFocusable", false),
                search.prefer_current,
            )
            .map(|next| (next, search.buffer.as_str()))
        })
    };
    if let Some((next, buffer)) = matched {
        write_typeahead_state(state, buffer);
        state.flags.focused = true;
        super::super::set_value(state, "focused_index".to_string(), UiValue::Int(next));
        return Ok(());
    }

    if let Some(search) = searches.first() {
        write_typeahead_state(state, &search.buffer);
    }
    Ok(())
}

pub(super) fn apply_typeahead_expired(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    if !is_focus_control(descriptor) {
        return Ok(());
    }
    super::super::set_value(
        state,
        MENU_TYPEAHEAD_BUFFER_EXPIRED.to_string(),
        UiValue::Bool(true),
    );
    Ok(())
}

pub(super) fn is_focus_control(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "menu" | "menu-list" | "context-menu" | "dropdown-popup"
    ) || matches!(
        descriptor.id.as_str(),
        "Menu"
            | "MenuList"
            | "PopupMenu"
            | "MenuPopup"
            | "ContextMenu"
            | "ContextActionMenu"
            | "DropdownPopup"
    )
}

pub(super) fn sync_search_filter(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    changed_property: &str,
) -> Result<(), UiComponentEventError> {
    if !is_focus_control(descriptor) {
        return Ok(());
    }

    if is_search_filter_property(changed_property) {
        sync_search_filter_state(state, descriptor)?;
    }
    if submenu::is_submenu_state_property(changed_property) {
        submenu::sync_submenu_state(state, descriptor, changed_property)?;
    }
    Ok(())
}

fn sync_search_filter_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    let search_options = menu_search_options(state, descriptor);
    let all_ids = all_search_option_ids(&search_options);

    if !allow_search(state, descriptor) {
        write_filter_state(state, &all_ids, false);
        return Ok(());
    }

    let eligibility = super::ExplicitOptionEligibility::new(state, descriptor);
    let search = search_query(state, descriptor);
    let (filtered_ids, focus_candidates) = match search.as_deref() {
        Some(query) => recursive_search_filter(&search_options, query),
        None => (
            all_ids,
            search_options
                .iter()
                .filter(|option| option.default_focus_candidate)
                .map(|option| MenuSearchFocusCandidate {
                    top_level_index: option.top_level_index,
                    top_level_id: option.id.clone(),
                    option_id: option.id.clone(),
                })
                .collect(),
        ),
    };
    let no_results = search.is_some() && filtered_ids.is_empty();
    let focus_index = next_search_focus_index(state, descriptor, &focus_candidates, &eligibility);

    write_filter_state(state, &filtered_ids, no_results);
    super::super::set_value(
        state,
        "focused_index".to_string(),
        UiValue::Int(focus_index),
    );
    state.flags.focused = focus_index >= 0;
    Ok(())
}

pub(super) struct MenuSearchFilter<'a> {
    filtered_ids: HashSet<&'a str>,
    no_results: bool,
}

pub(super) fn option_search_filter<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
) -> Option<MenuSearchFilter<'a>> {
    if !is_focus_control(descriptor) || !search_query_active(state, descriptor) {
        return None;
    }

    let mut filtered_ids = HashSet::new();
    if let Some(value) = state.values.get(MENU_FILTERED_OPTION_IDS) {
        collect_filtered_option_id_refs(value, &mut filtered_ids);
    }
    Some(MenuSearchFilter {
        filtered_ids,
        no_results: super::bool_setting(state, descriptor, MENU_FILTER_NO_RESULTS, false),
    })
}

pub(super) fn option_is_hidden_by_search_filter(
    filter: &MenuSearchFilter<'_>,
    option_id: &str,
) -> bool {
    if filter.filtered_ids.is_empty() {
        return filter.no_results;
    }
    !filter.filtered_ids.contains(option_id)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MenuTypeaheadSearch {
    buffer: String,
    prefer_current: bool,
}

fn menu_typeahead_searches(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: &str,
) -> Option<Vec<MenuTypeaheadSearch>> {
    let payload = keyboard_text_search(text)?;
    if payload.chars().nth(1).is_some() {
        return Some(vec![MenuTypeaheadSearch {
            buffer: payload,
            prefer_current: true,
        }]);
    }

    let previous = if super::bool_setting(state, descriptor, MENU_TYPEAHEAD_BUFFER_EXPIRED, false) {
        String::new()
    } else {
        super::string_setting(state, descriptor, MENU_TYPEAHEAD_BUFFER)
            .and_then(|buffer| keyboard_text_search(&buffer))
            .unwrap_or_default()
    };
    let mut combined = previous;
    combined.push_str(&payload);
    if repeated_typeahead_character(&combined) {
        return Some(vec![MenuTypeaheadSearch {
            buffer: payload,
            prefer_current: false,
        }]);
    }
    let prefer_current = combined.chars().nth(1).is_some();
    let needs_fallback = combined != payload;
    let mut searches = vec![MenuTypeaheadSearch {
        buffer: combined,
        prefer_current,
    }];
    if needs_fallback {
        searches.push(MenuTypeaheadSearch {
            buffer: payload,
            prefer_current: false,
        });
    }
    Some(searches)
}

fn keyboard_text_search(text: &str) -> Option<String> {
    let mut search = String::new();
    let mut trailing_whitespace_start = None;
    for ch in text.chars() {
        if ch.is_control() || (search.is_empty() && ch.is_whitespace()) {
            continue;
        }
        if ch.is_whitespace() {
            trailing_whitespace_start.get_or_insert(search.len());
        } else {
            trailing_whitespace_start = None;
        }
        search.extend(ch.to_lowercase());
    }
    if let Some(start) = trailing_whitespace_start {
        search.truncate(start);
    }
    (!search.is_empty()).then_some(search)
}

fn repeated_typeahead_character(search: &str) -> bool {
    let mut chars = search.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    chars.all(|ch| ch == first)
}

fn write_typeahead_state(state: &mut UiComponentState, buffer: &str) {
    super::super::set_value(
        state,
        MENU_TYPEAHEAD_BUFFER.to_string(),
        UiValue::String(buffer.to_string()),
    );
    super::super::set_value(
        state,
        MENU_TYPEAHEAD_BUFFER_EXPIRED.to_string(),
        UiValue::Bool(false),
    );
}

fn next_text_match_index(
    eligibility: &super::OptionEligibility<'_>,
    current: i64,
    options: &[super::OptionEntry],
    search: &str,
    wrap: bool,
    skip_disabled: bool,
    prefer_current: bool,
) -> Option<i64> {
    let max_index = (options.len() - 1) as i64;
    let current = current.clamp(0, max_index);
    let focusable =
        |index: i64| !skip_disabled || !eligibility.is_disabled(&options[index as usize].id);
    let matches =
        |index: i64| focusable(index) && option_text_matches(&options[index as usize].text, search);

    if prefer_current && matches(current) {
        return Some(current);
    }

    ((current + 1)..=max_index)
        .find(|index| matches(*index))
        .or_else(|| {
            wrap.then(|| (0..=current).find(|index| matches(*index)))
                .flatten()
        })
        .or_else(|| (!prefer_current && matches(current)).then_some(current))
}

fn option_text_matches(text: &str, search: &str) -> bool {
    starts_with_lowercase_query(text, search)
}

fn is_search_filter_property(property: &str) -> bool {
    matches!(
        property,
        MENU_SEARCH_QUERY
            | "options"
            | "disabled_options"
            | "disabledItemsFocusable"
            | MENU_ALLOW_SEARCH
            | MENU_ALLOW_SEARCH_CAMEL
            | MENU_SEARCH_BAR_ENABLED_ON_ITEM_COUNT
            | MENU_SEARCH_BAR_ENABLED_ON_ITEM_COUNT_CAMEL
    )
}

fn allow_search(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    let camel_default = super::bool_setting(state, descriptor, MENU_ALLOW_SEARCH_CAMEL, true);
    super::bool_setting(state, descriptor, MENU_ALLOW_SEARCH, camel_default)
}

fn search_query(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> Option<String> {
    let query = nonempty_string_setting_ref(state, descriptor, MENU_SEARCH_QUERY)?;
    let query = query.trim();
    (!query.is_empty()).then(|| query.to_lowercase())
}

fn search_query_active(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    string_setting_ref(state, descriptor, MENU_SEARCH_QUERY)
        .is_some_and(|query| !query.trim().is_empty())
}

fn string_setting_ref<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
    property: &str,
) -> Option<&'a str> {
    state
        .values
        .get(property)
        .and_then(string_value_ref)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(string_value_ref)
        })
}

fn nonempty_string_setting_ref<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
    property: &str,
) -> Option<&'a str> {
    state
        .values
        .get(property)
        .and_then(nonempty_string_value_ref)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(nonempty_string_value_ref)
        })
}

fn string_value_ref(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.as_str()),
        _ => None,
    }
}

fn nonempty_string_value_ref(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => Some(value.as_str()),
        _ => None,
    }
}

fn collect_filtered_option_id_refs<'a>(value: &'a UiValue, ids: &mut HashSet<&'a str>) {
    match value {
        UiValue::Array(values) => {
            ids.reserve(values.len());
            for value in values {
                collect_filtered_option_id_refs(value, ids);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => {
            ids.insert(value.as_str());
        }
        UiValue::Flags(values) => {
            ids.reserve(values.len());
            ids.extend(values.iter().map(String::as_str));
        }
        _ => {}
    }
}

fn option_text_or_id_matches_search(id: &str, text: &str, query: &str) -> bool {
    contains_lowercase_query(text, query) || contains_lowercase_query(id, query)
}

fn write_filter_state(state: &mut UiComponentState, ids: &[String], no_results: bool) {
    super::super::set_value(
        state,
        MENU_FILTERED_OPTION_IDS.to_string(),
        UiValue::Array(ids.iter().cloned().map(UiValue::String).collect()),
    );
    super::super::set_value(
        state,
        MENU_FILTER_NO_RESULTS.to_string(),
        UiValue::Bool(no_results),
    );
}

fn next_search_focus_index(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    candidates: &[MenuSearchFocusCandidate],
    eligibility: &super::ExplicitOptionEligibility<'_>,
) -> i64 {
    if candidates.is_empty() {
        return -1;
    }

    let skip_disabled = !super::bool_setting(state, descriptor, "disabledItemsFocusable", false);
    let is_focusable = |candidate: &MenuSearchFocusCandidate| {
        !skip_disabled
            || (!eligibility.is_disabled(&candidate.top_level_id)
                && !eligibility.is_disabled(&candidate.option_id))
    };
    let current = super::int_setting(state, descriptor, "focused_index").unwrap_or(-1);
    if candidates
        .iter()
        .any(|candidate| candidate.top_level_index == current && is_focusable(candidate))
    {
        return current;
    }

    candidates
        .iter()
        .find(|candidate| is_focusable(candidate))
        .map(|candidate| candidate.top_level_index)
        .unwrap_or(-1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MenuSearchOption {
    id: String,
    text: String,
    top_level_index: i64,
    top_level_id: String,
    default_focus_candidate: bool,
    children: Vec<MenuSearchOption>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MenuSearchFocusCandidate {
    top_level_index: i64,
    top_level_id: String,
    option_id: String,
}

#[derive(Default)]
struct MenuSearchBuild {
    filtered_ids: Vec<String>,
    focus_candidates: Vec<MenuSearchFocusCandidate>,
}

fn menu_search_options(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Vec<MenuSearchOption> {
    state
        .values
        .get("options")
        .or_else(|| {
            descriptor
                .prop("options")
                .and_then(|schema| schema.default_value.as_ref())
        })
        .map(menu_search_option_list)
        .unwrap_or_default()
}

fn menu_search_option_list(value: &UiValue) -> Vec<MenuSearchOption> {
    let mut next_top_level_index = 0;
    let mut options = Vec::with_capacity(menu_search_option_capacity_hint(value));
    collect_top_level_search_options(value, &mut next_top_level_index, true, &mut options);
    options
}

fn menu_search_option_capacity_hint(value: &UiValue) -> usize {
    match value {
        UiValue::Array(values) => values.len(),
        UiValue::String(value) | UiValue::Enum(value) => {
            if value.is_empty() {
                0
            } else {
                1
            }
        }
        UiValue::Map(_) => 1,
        _ => 0,
    }
}

fn collect_top_level_search_options(
    value: &UiValue,
    next_top_level_index: &mut i64,
    default_focus_candidate: bool,
    options: &mut Vec<MenuSearchOption>,
) {
    match value {
        UiValue::Array(values) => {
            options.reserve(values.len());
            for value in values {
                collect_top_level_search_options(
                    value,
                    next_top_level_index,
                    default_focus_candidate,
                    options,
                );
            }
        }
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => {
            options.push(MenuSearchOption {
                id: value.clone(),
                text: value.clone(),
                top_level_index: *next_top_level_index,
                top_level_id: value.clone(),
                default_focus_candidate,
                children: Vec::new(),
            });
            *next_top_level_index += 1;
        }
        UiValue::Map(values) => {
            let ids = menu_option_ids(values);
            let text = menu_option_label_text(values);
            if ids.is_empty() {
                for value in menu_child_values(values) {
                    collect_top_level_search_options(value, next_top_level_index, false, options);
                }
                return;
            }

            for id in ids {
                if id.is_empty() {
                    continue;
                }
                let top_level_index = *next_top_level_index;
                *next_top_level_index += 1;
                options.push(MenuSearchOption {
                    children: collect_child_search_options(values, top_level_index, &id),
                    text: text.map(str::to_owned).unwrap_or_else(|| id.clone()),
                    top_level_id: id.clone(),
                    top_level_index,
                    default_focus_candidate,
                    id,
                });
            }
        }
        _ => {}
    }
}

fn collect_child_search_options(
    values: &std::collections::BTreeMap<String, UiValue>,
    top_level_index: i64,
    top_level_id: &str,
) -> Vec<MenuSearchOption> {
    let mut children = Vec::new();
    for value in menu_child_values(values) {
        collect_descendant_search_options(value, top_level_index, top_level_id, &mut children);
    }
    children
}

fn collect_descendant_search_options(
    value: &UiValue,
    top_level_index: i64,
    top_level_id: &str,
    options: &mut Vec<MenuSearchOption>,
) {
    match value {
        UiValue::Array(values) => {
            options.reserve(values.len());
            for value in values {
                collect_descendant_search_options(value, top_level_index, top_level_id, options);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => {
            options.push(MenuSearchOption {
                id: value.clone(),
                text: value.clone(),
                top_level_index,
                top_level_id: top_level_id.to_string(),
                default_focus_candidate: false,
                children: Vec::new(),
            });
        }
        UiValue::Map(values) => {
            let ids = menu_option_ids(values);
            let text = menu_option_label_text(values);
            if ids.is_empty() {
                for value in menu_child_values(values) {
                    collect_descendant_search_options(
                        value,
                        top_level_index,
                        top_level_id,
                        options,
                    );
                }
                return;
            }

            for id in ids {
                if id.is_empty() {
                    continue;
                }
                options.push(MenuSearchOption {
                    children: collect_child_search_options(values, top_level_index, top_level_id),
                    text: text.map(str::to_owned).unwrap_or_else(|| id.clone()),
                    top_level_id: top_level_id.to_string(),
                    top_level_index,
                    default_focus_candidate: false,
                    id,
                });
            }
        }
        _ => {}
    }
}

fn menu_option_ids(values: &std::collections::BTreeMap<String, UiValue>) -> Vec<String> {
    values
        .get("id")
        .or_else(|| values.get("value"))
        .or_else(|| values.get("row_id"))
        .or_else(|| values.get("rowId"))
        .or_else(|| values.get("node_id"))
        .or_else(|| values.get("nodeId"))
        .or_else(|| values.get("key"))
        .map(super::option_id_list)
        .unwrap_or_default()
}

fn menu_option_label_text<'a>(
    values: &'a std::collections::BTreeMap<String, UiValue>,
) -> Option<&'a str> {
    ["label", "text", "title", "value_text", "value", "id"]
        .into_iter()
        .filter_map(|property| values.get(property).and_then(string_value_ref))
        .find(|value| !value.is_empty())
}

fn menu_child_values<'a>(
    values: &'a std::collections::BTreeMap<String, UiValue>,
) -> impl Iterator<Item = &'a UiValue> {
    MENU_CHILD_PROPERTY_NAMES
        .iter()
        .filter_map(move |property| values.get(*property))
}

fn all_search_option_ids(options: &[MenuSearchOption]) -> Vec<String> {
    let mut ids = Vec::with_capacity(options.len());
    for option in options {
        collect_search_option_ids(option, &mut ids);
    }
    ids
}

fn collect_search_option_ids(option: &MenuSearchOption, ids: &mut Vec<String>) {
    ids.push(option.id.clone());
    ids.reserve(option.children.len());
    for child in &option.children {
        collect_search_option_ids(child, ids);
    }
}

fn recursive_search_filter(
    options: &[MenuSearchOption],
    query: &str,
) -> (Vec<String>, Vec<MenuSearchFocusCandidate>) {
    let mut filter = MenuSearchBuild::default();
    for option in options {
        collect_matching_search_options(option, query, &mut filter);
    }
    (filter.filtered_ids, filter.focus_candidates)
}

fn collect_matching_search_options(
    option: &MenuSearchOption,
    query: &str,
    filter: &mut MenuSearchBuild,
) -> bool {
    let ids_start = filter.filtered_ids.len();
    let focus_start = filter.focus_candidates.len();
    let matches = option_text_or_id_matches_search(&option.id, &option.text, query);
    if !matches && option.children.is_empty() {
        return false;
    }

    filter.filtered_ids.push(option.id.clone());
    if matches {
        filter.focus_candidates.push(MenuSearchFocusCandidate {
            top_level_index: option.top_level_index,
            top_level_id: option.top_level_id.clone(),
            option_id: option.id.clone(),
        });
    }

    for child in &option.children {
        collect_matching_search_options(child, query, filter);
    }

    if !matches && filter.filtered_ids.len() == ids_start + 1 {
        filter.filtered_ids.truncate(ids_start);
        filter.focus_candidates.truncate(focus_start);
        return false;
    }
    true
}
