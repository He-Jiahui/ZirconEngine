use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValue,
};

const MENU_HOVERED_OPTION_ID: &str = "hovered_option_id";
const MENU_SUBMENU_ACTIVE_PARENT_INDEX: &str = "submenu_active_parent_index";
const MENU_SUBMENU_FOCUS_LOOP: &str = "submenu_focus_loop";
const MENU_SUBMENU_FOCUS_SCOPE: &str = "submenu_focus_scope";
const MENU_SUBMENU_FOCUS_SCOPE_ROOT: &str = "root";
const MENU_SUBMENU_FOCUS_SCOPE_SUBMENU: &str = "submenu";
const MENU_SUBMENU_HOVER_READY: &str = "submenu_hover_ready";
const MENU_SUBMENU_OPEN_OPTION_ID: &str = "submenu_open_option_id";
const MENU_SUBMENU_PENDING_OPTION_ID: &str = "submenu_pending_option_id";

pub(in crate::ui::component::state_reducer::keyboard) fn open_focused_submenu(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> bool {
    if !super::is_focus_control(descriptor)
        || !super::super::bool_setting(state, descriptor, MENU_SUBMENU_FOCUS_LOOP, true)
    {
        return false;
    }

    let options = super::super::option_entries(state, descriptor);
    if options.is_empty() {
        return false;
    }

    let option_ids = options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>();
    let current = super::super::current_index(state, descriptor, &option_ids)
        .clamp(0, (options.len() - 1) as i64);
    let option_id = &options[current as usize].id;
    open_submenu_for_option_id(state, descriptor, option_id, current)
}

pub(in crate::ui::component::state_reducer::keyboard) fn close_active_submenu(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> bool {
    if !super::is_focus_control(descriptor)
        || !super::super::bool_setting(state, descriptor, MENU_SUBMENU_FOCUS_LOOP, true)
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

pub(super) fn sync_submenu_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    changed_property: &str,
) -> Result<(), UiComponentEventError> {
    if !super::super::bool_setting(state, descriptor, MENU_SUBMENU_FOCUS_LOOP, true) {
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

pub(super) fn is_submenu_state_property(property: &str) -> bool {
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
    if !super::super::bool_setting(state, descriptor, MENU_SUBMENU_HOVER_READY, false) {
        return;
    }

    let Some(pending_option_id) = submenu_string(state, descriptor, MENU_SUBMENU_PENDING_OPTION_ID)
    else {
        super::super::super::set_value(
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
    super::super::super::set_value(
        state,
        MENU_SUBMENU_ACTIVE_PARENT_INDEX.to_string(),
        UiValue::Int(parent_index),
    );
    super::super::super::set_value(
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
    super::super::super::set_value(
        state,
        MENU_SUBMENU_ACTIVE_PARENT_INDEX.to_string(),
        UiValue::Int(parent_index),
    );
    super::super::super::set_value(
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
    super::super::super::set_value(
        state,
        MENU_SUBMENU_ACTIVE_PARENT_INDEX.to_string(),
        UiValue::Int(-1),
    );
    super::super::super::set_value(
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
    if option_id.is_empty() || super::super::option_is_disabled(state, descriptor, option_id) {
        return None;
    }

    find_submenu_option_target(&super::menu_search_options(state, descriptor), option_id)
}

fn find_submenu_option_target(
    options: &[super::MenuSearchOption],
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
    super::super::string_setting(state, descriptor, property)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn write_submenu_string(state: &mut UiComponentState, property: &str, value: &str) {
    super::super::super::set_value(
        state,
        property.to_string(),
        UiValue::String(value.to_string()),
    );
}
