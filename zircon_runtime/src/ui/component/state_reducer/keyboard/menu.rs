use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValue,
};

const MENU_TYPEAHEAD_BUFFER: &str = "typeahead_buffer";
const MENU_TYPEAHEAD_BUFFER_EXPIRED: &str = "typeahead_buffer_expired";
const MENU_ALLOW_SEARCH: &str = "allow_search";
const MENU_ALLOW_SEARCH_CAMEL: &str = "allowSearch";
const MENU_FILTERED_OPTION_IDS: &str = "filtered_option_ids";
const MENU_FILTER_NO_RESULTS: &str = "filter_no_results";
const MENU_HOVERED_OPTION_ID: &str = "hovered_option_id";
const MENU_SEARCH_BAR_ENABLED_ON_ITEM_COUNT: &str = "search_bar_enabled_on_item_count";
const MENU_SEARCH_BAR_ENABLED_ON_ITEM_COUNT_CAMEL: &str = "searchBarEnabledOnItemCount";
const MENU_SEARCH_QUERY: &str = "search_query";
const MENU_SUBMENU_ACTIVE_PARENT_INDEX: &str = "submenu_active_parent_index";
const MENU_SUBMENU_FOCUS_LOOP: &str = "submenu_focus_loop";
const MENU_SUBMENU_FOCUS_SCOPE: &str = "submenu_focus_scope";
const MENU_SUBMENU_FOCUS_SCOPE_ROOT: &str = "root";
const MENU_SUBMENU_FOCUS_SCOPE_SUBMENU: &str = "submenu";
const MENU_SUBMENU_HOVER_READY: &str = "submenu_hover_ready";
const MENU_SUBMENU_OPEN_OPTION_ID: &str = "submenu_open_option_id";
const MENU_SUBMENU_PENDING_OPTION_ID: &str = "submenu_pending_option_id";

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

    let option_ids = options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>();
    let current = super::current_index(state, descriptor, &option_ids);
    for search in &searches {
        let Some(next) = next_text_match_index(
            state,
            descriptor,
            current,
            &options,
            &search.search,
            !super::bool_setting(state, descriptor, "disableListWrap", false),
            !super::bool_setting(state, descriptor, "disabledItemsFocusable", false),
            search.prefer_current,
        ) else {
            continue;
        };

        write_typeahead_state(state, &search.buffer);
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
    matches!(descriptor.role.as_str(), "menu" | "menu-list")
        || matches!(descriptor.id.as_str(), "Menu" | "MenuList")
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
    if is_submenu_state_property(changed_property) {
        sync_submenu_state(state, descriptor, changed_property)?;
    }
    Ok(())
}

pub(super) fn open_focused_submenu(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> bool {
    if !is_focus_control(descriptor)
        || !super::bool_setting(state, descriptor, MENU_SUBMENU_FOCUS_LOOP, true)
    {
        return false;
    }

    let options = super::option_entries(state, descriptor);
    if options.is_empty() {
        return false;
    }

    let option_ids = options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>();
    let current =
        super::current_index(state, descriptor, &option_ids).clamp(0, (options.len() - 1) as i64);
    let option_id = &options[current as usize].id;
    open_submenu_for_option_id(state, descriptor, option_id, current)
}

pub(super) fn close_active_submenu(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> bool {
    if !is_focus_control(descriptor)
        || !super::bool_setting(state, descriptor, MENU_SUBMENU_FOCUS_LOOP, true)
    {
        return false;
    }

    let open = submenu_string(state, descriptor, MENU_SUBMENU_OPEN_OPTION_ID)
        .is_some_and(|option_id| !option_id.is_empty());
    let pending = submenu_string(state, descriptor, MENU_SUBMENU_PENDING_OPTION_ID)
        .is_some_and(|option_id| !option_id.is_empty());
    let in_submenu = submenu_string(state, descriptor, MENU_SUBMENU_FOCUS_SCOPE)
        .is_some_and(|scope| scope == MENU_SUBMENU_FOCUS_SCOPE_SUBMENU);
    if !open && !pending && !in_submenu {
        return false;
    }

    clear_submenu_state(state);
    true
}

fn sync_search_filter_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    let options = super::option_entries(state, descriptor);
    let search_options = menu_search_options(state, descriptor);
    let all_ids = all_search_option_ids(&search_options);

    if !allow_search(state, descriptor) {
        write_filter_state(state, &all_ids, false);
        return Ok(());
    }

    let search = search_query(state, descriptor);
    let (filtered_ids, focus_candidates) = match search.as_deref() {
        Some(query) => recursive_search_filter(&search_options, query),
        None => (
            all_ids,
            options
                .iter()
                .enumerate()
                .map(|(index, option)| MenuSearchFocusCandidate {
                    top_level_index: index as i64,
                    top_level_id: option.id.clone(),
                    option_id: option.id.clone(),
                })
                .collect(),
        ),
    };
    let no_results = search.is_some() && filtered_ids.is_empty();

    write_filter_state(state, &filtered_ids, no_results);

    let focus_index = next_search_focus_index(state, descriptor, &focus_candidates);
    super::super::set_value(
        state,
        "focused_index".to_string(),
        UiValue::Int(focus_index),
    );
    state.flags.focused = focus_index >= 0;
    Ok(())
}

fn sync_submenu_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    changed_property: &str,
) -> Result<(), UiComponentEventError> {
    if !super::bool_setting(state, descriptor, MENU_SUBMENU_FOCUS_LOOP, true) {
        clear_submenu_state(state);
        return Ok(());
    }

    match changed_property {
        MENU_HOVERED_OPTION_ID => sync_hovered_submenu_option(state, descriptor),
        MENU_SUBMENU_HOVER_READY => promote_pending_submenu_if_ready(state, descriptor),
        "options" | "disabled_options" | "disabledItemsFocusable" => {
            prune_invalid_submenu_state(state, descriptor)
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn option_is_hidden_by_search_filter(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    option_id: &str,
) -> bool {
    if !is_focus_control(descriptor) || search_query(state, descriptor).is_none() {
        return false;
    }

    let filtered_ids = state
        .values
        .get(MENU_FILTERED_OPTION_IDS)
        .map(super::option_id_list)
        .unwrap_or_default();
    if filtered_ids.is_empty() {
        return super::bool_setting(state, descriptor, MENU_FILTER_NO_RESULTS, false);
    }

    !filtered_ids.iter().any(|id| id == option_id)
}

fn is_submenu_state_property(property: &str) -> bool {
    matches!(
        property,
        MENU_HOVERED_OPTION_ID
            | MENU_SUBMENU_HOVER_READY
            | "options"
            | "disabled_options"
            | "disabledItemsFocusable"
    )
}

fn sync_hovered_submenu_option(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    let Some(hovered_option_id) = submenu_string(state, descriptor, MENU_HOVERED_OPTION_ID) else {
        clear_submenu_state(state);
        return;
    };

    let Some((option_id, parent_index)) =
        submenu_target_for_option_id(state, descriptor, &hovered_option_id)
    else {
        clear_submenu_state(state);
        return;
    };

    write_pending_submenu_state(state, &option_id, parent_index);
}

fn promote_pending_submenu_if_ready(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) {
    if !super::bool_setting(state, descriptor, MENU_SUBMENU_HOVER_READY, false) {
        return;
    }

    let Some(pending_option_id) = submenu_string(state, descriptor, MENU_SUBMENU_PENDING_OPTION_ID)
    else {
        super::super::set_value(
            state,
            MENU_SUBMENU_HOVER_READY.to_string(),
            UiValue::Bool(false),
        );
        return;
    };

    if !open_submenu_for_option_id(state, descriptor, &pending_option_id, -1) {
        clear_submenu_state(state);
    }
}

fn prune_invalid_submenu_state(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    let invalid_open =
        submenu_string(state, descriptor, MENU_SUBMENU_OPEN_OPTION_ID).is_some_and(|option_id| {
            submenu_target_for_option_id(state, descriptor, &option_id).is_none()
        });
    let invalid_pending = submenu_string(state, descriptor, MENU_SUBMENU_PENDING_OPTION_ID)
        .is_some_and(|option_id| {
            submenu_target_for_option_id(state, descriptor, &option_id).is_none()
        });
    if invalid_open || invalid_pending {
        clear_submenu_state(state);
    }
}

fn open_submenu_for_option_id(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    option_id: &str,
    requested_parent_index: i64,
) -> bool {
    let Some((option_id, parent_index)) =
        submenu_target_for_option_id(state, descriptor, option_id)
    else {
        return false;
    };
    let parent_index = if requested_parent_index >= 0 {
        requested_parent_index
    } else {
        parent_index
    };

    write_submenu_string(state, MENU_SUBMENU_OPEN_OPTION_ID, &option_id);
    write_submenu_string(state, MENU_SUBMENU_PENDING_OPTION_ID, "");
    write_submenu_string(
        state,
        MENU_SUBMENU_FOCUS_SCOPE,
        MENU_SUBMENU_FOCUS_SCOPE_SUBMENU,
    );
    super::super::set_value(
        state,
        MENU_SUBMENU_ACTIVE_PARENT_INDEX.to_string(),
        UiValue::Int(parent_index),
    );
    super::super::set_value(
        state,
        MENU_SUBMENU_HOVER_READY.to_string(),
        UiValue::Bool(false),
    );
    state.flags.focused = true;
    true
}

fn write_pending_submenu_state(state: &mut UiComponentState, option_id: &str, parent_index: i64) {
    write_submenu_string(state, MENU_SUBMENU_PENDING_OPTION_ID, option_id);
    write_submenu_string(state, MENU_SUBMENU_OPEN_OPTION_ID, "");
    write_submenu_string(
        state,
        MENU_SUBMENU_FOCUS_SCOPE,
        MENU_SUBMENU_FOCUS_SCOPE_ROOT,
    );
    super::super::set_value(
        state,
        MENU_SUBMENU_ACTIVE_PARENT_INDEX.to_string(),
        UiValue::Int(parent_index),
    );
    super::super::set_value(
        state,
        MENU_SUBMENU_HOVER_READY.to_string(),
        UiValue::Bool(false),
    );
}

fn clear_submenu_state(state: &mut UiComponentState) {
    write_submenu_string(state, MENU_SUBMENU_PENDING_OPTION_ID, "");
    write_submenu_string(state, MENU_SUBMENU_OPEN_OPTION_ID, "");
    write_submenu_string(
        state,
        MENU_SUBMENU_FOCUS_SCOPE,
        MENU_SUBMENU_FOCUS_SCOPE_ROOT,
    );
    super::super::set_value(
        state,
        MENU_SUBMENU_ACTIVE_PARENT_INDEX.to_string(),
        UiValue::Int(-1),
    );
    super::super::set_value(
        state,
        MENU_SUBMENU_HOVER_READY.to_string(),
        UiValue::Bool(false),
    );
}

fn submenu_target_for_option_id(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    option_id: &str,
) -> Option<(String, i64)> {
    let option_id = option_id.trim();
    if option_id.is_empty() || super::option_is_disabled(state, descriptor, option_id) {
        return None;
    }

    find_submenu_option_target(&menu_search_options(state, descriptor), option_id)
}

fn find_submenu_option_target(
    options: &[MenuSearchOption],
    option_id: &str,
) -> Option<(String, i64)> {
    for option in options {
        if option.id == option_id && !option.children.is_empty() {
            return Some((option.id.clone(), option.top_level_index));
        }
        if let Some(target) = find_submenu_option_target(&option.children, option_id) {
            return Some(target);
        }
    }
    None
}

fn submenu_string(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    super::string_setting(state, descriptor, property)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn write_submenu_string(state: &mut UiComponentState, property: &str, value: &str) {
    super::super::set_value(
        state,
        property.to_string(),
        UiValue::String(value.to_string()),
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MenuTypeaheadSearch {
    search: String,
    buffer: String,
    prefer_current: bool,
}

fn menu_typeahead_searches(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: &str,
) -> Option<Vec<MenuTypeaheadSearch>> {
    let payload = keyboard_text_search(text)?;
    if payload.chars().count() > 1 {
        return Some(vec![MenuTypeaheadSearch {
            search: payload.clone(),
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
    let combined = format!("{previous}{payload}");
    let buffer = if repeated_typeahead_character(&combined) {
        payload.clone()
    } else {
        combined
    };
    let mut searches = vec![MenuTypeaheadSearch {
        search: buffer.clone(),
        buffer: buffer.clone(),
        prefer_current: buffer.chars().count() > 1,
    }];
    if buffer != payload {
        searches.push(MenuTypeaheadSearch {
            search: payload.clone(),
            buffer: payload,
            prefer_current: false,
        });
    }
    Some(searches)
}

fn keyboard_text_search(text: &str) -> Option<String> {
    let search = text
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>()
        .trim()
        .to_lowercase();
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
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    current: i64,
    options: &[super::OptionEntry],
    search: &str,
    wrap: bool,
    skip_disabled: bool,
    prefer_current: bool,
) -> Option<i64> {
    let max_index = (options.len() - 1) as i64;
    let current = current.clamp(0, max_index);
    let focusable = |index: i64| {
        !skip_disabled || !super::option_is_disabled(state, descriptor, &options[index as usize].id)
    };
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
    text.trim_start().to_lowercase().starts_with(search)
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
    super::string_setting(state, descriptor, MENU_SEARCH_QUERY)
        .map(|query| query.trim().to_lowercase())
        .filter(|query| !query.is_empty())
}

fn option_text_or_id_matches_search(id: &str, text: &str, query: &str) -> bool {
    text.trim().to_lowercase().contains(query) || id.trim().to_lowercase().contains(query)
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
) -> i64 {
    if candidates.is_empty() {
        return -1;
    }

    let skip_disabled = !super::bool_setting(state, descriptor, "disabledItemsFocusable", false);
    let is_focusable = |candidate: &MenuSearchFocusCandidate| {
        !skip_disabled
            || (!super::option_is_explicitly_disabled(state, descriptor, &candidate.top_level_id)
                && !super::option_is_explicitly_disabled(state, descriptor, &candidate.option_id))
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
    children: Vec<MenuSearchOption>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MenuSearchFocusCandidate {
    top_level_index: i64,
    top_level_id: String,
    option_id: String,
}

#[derive(Default)]
struct MenuSearchFilter {
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
    collect_top_level_search_options(value, &mut next_top_level_index)
}

fn collect_top_level_search_options(
    value: &UiValue,
    next_top_level_index: &mut i64,
) -> Vec<MenuSearchOption> {
    match value {
        UiValue::Array(values) => values
            .iter()
            .flat_map(|value| collect_top_level_search_options(value, next_top_level_index))
            .collect(),
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => {
            let option = MenuSearchOption {
                id: value.clone(),
                text: value.clone(),
                top_level_index: *next_top_level_index,
                top_level_id: value.clone(),
                children: Vec::new(),
            };
            *next_top_level_index += 1;
            vec![option]
        }
        UiValue::Map(values) => {
            let ids = menu_option_ids(values);
            let text = menu_option_label_text(values);
            if ids.is_empty() {
                return menu_child_values(values)
                    .into_iter()
                    .flat_map(|value| collect_top_level_search_options(value, next_top_level_index))
                    .collect();
            }

            ids.into_iter()
                .filter(|id| !id.is_empty())
                .map(|id| {
                    let top_level_index = *next_top_level_index;
                    *next_top_level_index += 1;
                    MenuSearchOption {
                        children: collect_child_search_options(values, top_level_index, &id),
                        text: text.clone().unwrap_or_else(|| id.clone()),
                        top_level_id: id.clone(),
                        top_level_index,
                        id,
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn collect_child_search_options(
    values: &std::collections::BTreeMap<String, UiValue>,
    top_level_index: i64,
    top_level_id: &str,
) -> Vec<MenuSearchOption> {
    menu_child_values(values)
        .into_iter()
        .flat_map(|value| {
            collect_descendant_search_options(value, top_level_index, top_level_id.to_string())
        })
        .collect()
}

fn collect_descendant_search_options(
    value: &UiValue,
    top_level_index: i64,
    top_level_id: String,
) -> Vec<MenuSearchOption> {
    match value {
        UiValue::Array(values) => values
            .iter()
            .flat_map(|value| {
                collect_descendant_search_options(value, top_level_index, top_level_id.clone())
            })
            .collect(),
        UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => {
            vec![MenuSearchOption {
                id: value.clone(),
                text: value.clone(),
                top_level_index,
                top_level_id,
                children: Vec::new(),
            }]
        }
        UiValue::Map(values) => {
            let ids = menu_option_ids(values);
            let text = menu_option_label_text(values);
            if ids.is_empty() {
                return menu_child_values(values)
                    .into_iter()
                    .flat_map(|value| {
                        collect_descendant_search_options(
                            value,
                            top_level_index,
                            top_level_id.clone(),
                        )
                    })
                    .collect();
            }

            ids.into_iter()
                .filter(|id| !id.is_empty())
                .map(|id| MenuSearchOption {
                    children: collect_child_search_options(values, top_level_index, &top_level_id),
                    text: text.clone().unwrap_or_else(|| id.clone()),
                    top_level_id: top_level_id.clone(),
                    top_level_index,
                    id,
                })
                .collect()
        }
        _ => Vec::new(),
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

fn menu_option_label_text(values: &std::collections::BTreeMap<String, UiValue>) -> Option<String> {
    ["label", "text", "title", "value_text", "value", "id"]
        .into_iter()
        .filter_map(|property| values.get(property).and_then(super::string_value))
        .find(|value| !value.is_empty())
}

fn menu_child_values(values: &std::collections::BTreeMap<String, UiValue>) -> Vec<&UiValue> {
    [
        "children", "items", "submenu", "sub_menu", "subMenu", "options",
    ]
    .into_iter()
    .filter_map(|property| values.get(property))
    .collect()
}

fn all_search_option_ids(options: &[MenuSearchOption]) -> Vec<String> {
    let mut ids = Vec::new();
    for option in options {
        collect_search_option_ids(option, &mut ids);
    }
    ids
}

fn collect_search_option_ids(option: &MenuSearchOption, ids: &mut Vec<String>) {
    ids.push(option.id.clone());
    for child in &option.children {
        collect_search_option_ids(child, ids);
    }
}

fn recursive_search_filter(
    options: &[MenuSearchOption],
    query: &str,
) -> (Vec<String>, Vec<MenuSearchFocusCandidate>) {
    let mut filter = MenuSearchFilter::default();
    for option in options {
        collect_matching_search_options(option, query, &mut filter);
    }
    (filter.filtered_ids, filter.focus_candidates)
}

fn collect_matching_search_options(
    option: &MenuSearchOption,
    query: &str,
    filter: &mut MenuSearchFilter,
) -> bool {
    let mut child_filter = MenuSearchFilter::default();
    for child in &option.children {
        collect_matching_search_options(child, query, &mut child_filter);
    }

    let matches = option_text_or_id_matches_search(&option.id, &option.text, query);
    if !matches && child_filter.filtered_ids.is_empty() {
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
    filter.filtered_ids.extend(child_filter.filtered_ids);
    filter
        .focus_candidates
        .extend(child_filter.focus_candidates);
    true
}
