use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        transfer_list(),
        context_menu(),
        pane_toolbar(),
        filter_bar(),
        severity_chips(),
        view_tab(),
        tab_stack(),
    ]
}

fn transfer_list() -> UiComponentDescriptor {
    composite(
        "TransferList",
        "Transfer List",
        UiComponentCategory::Collection,
        "transfer-list",
    )
    .with_prop(array_prop("source_items"))
    .with_prop(array_prop("sourceItems"))
    .with_prop(array_prop("target_items"))
    .with_prop(array_prop("targetItems"))
    .with_prop(array_prop("selected_items"))
    .with_prop(array_prop("selectedItems"))
    .with_prop(array_prop("source_selected_items"))
    .with_prop(array_prop("sourceSelectedItems"))
    .with_prop(array_prop("target_selected_items"))
    .with_prop(array_prop("targetSelectedItems"))
    .with_prop(array_prop("disabled_items"))
    .with_prop(array_prop("disabledItems"))
    .with_prop(array_prop("disabled_actions"))
    .with_prop(array_prop("disabledActions"))
    .slot(multi_slot("source"))
    .slot(multi_slot("target"))
    .slot(multi_slot("actions"))
    .events([
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::MoveElement,
    ])
}

fn context_menu() -> UiComponentDescriptor {
    overlay_layer_props(modal_interaction_props(popup_position_props(
        editor_panel_component(
            "ContextMenu",
            "Context Menu",
            UiComponentCategory::Input,
            "context-menu",
        )
        .with_prop(bool_prop("open", false))
        .with_prop(bool_prop("popup_open", false))
        .with_prop(options_prop())
        .with_prop(default_string_prop("context_target", ""))
        .with_prop(default_string_prop("context_target_path", ""))
        .with_prop(int_prop("focused_index", 0))
        .with_prop(array_prop("disabled_options"))
        .with_prop(bool_prop("keyboard_navigation", true))
        .with_prop(default_string_prop("typeahead_buffer", ""))
        .with_prop(bool_prop("typeahead_buffer_expired", false))
        .with_prop(int_prop("typeahead_timeout_ms", 500))
        .with_prop(default_string_prop("hovered_option_id", ""))
        .with_prop(default_string_prop("submenu_pending_option_id", ""))
        .with_prop(default_string_prop("submenu_open_option_id", ""))
        .with_prop(bool_prop("submenu_hover_ready", false))
        .with_prop(int_prop("submenu_hover_delay_ms", 300))
        .with_prop(default_string_prop("submenu_focus_scope", "root"))
        .with_prop(bool_prop("submenu_focus_loop", true)),
        "right-start",
    )))
    .slot(UiSlotSchema::new("paper"))
    .slot(UiSlotSchema::new("list"))
    .slot(UiSlotSchema::new("transition"))
    .slot(multi_slot("items"))
    .events([
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::KeyboardText,
        UiComponentEventKind::TypeaheadExpired,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Focus,
        UiComponentEventKind::OpenPopupAt,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::Commit,
    ])
}

fn pane_toolbar() -> UiComponentDescriptor {
    editor_panel_component(
        "PaneToolbar",
        "Pane Toolbar",
        UiComponentCategory::Container,
        "pane-toolbar",
    )
    .slot(multi_slot("actions"))
    .event(UiComponentEventKind::Commit)
}

fn filter_bar() -> UiComponentDescriptor {
    editor_panel_component(
        "FilterBar",
        "Filter Bar",
        UiComponentCategory::Input,
        "filter-bar",
    )
    .with_prop(string_prop("query"))
    .with_prop(enum_prop("severity", "all"))
    .slot(multi_slot("filters"))
    .events([
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::SelectOption,
    ])
}

fn severity_chips() -> UiComponentDescriptor {
    editor_panel_component(
        "SeverityChips",
        "Severity Chips",
        UiComponentCategory::Selection,
        "severity-chips",
    )
    .with_prop(enum_prop("selected_severity", "all"))
    .event(UiComponentEventKind::SelectOption)
}

fn view_tab() -> UiComponentDescriptor {
    shell("ViewTab", "View Tab", "view-tab")
        .with_prop(required_string_prop("view_id"))
        .with_prop(text_prop())
        .events([
            UiComponentEventKind::Commit,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::EndDrag,
        ])
}

fn tab_stack() -> UiComponentDescriptor {
    shell("TabStack", "Tab Stack", "tab-stack")
        .with_prop(string_prop("active_tab"))
        .slot(multi_slot("tabs"))
        .slot(multi_slot("content"))
        .event(UiComponentEventKind::SelectOption)
}

fn multi_slot(name: &str) -> UiSlotSchema {
    UiSlotSchema::new(name).multiple(true)
}
