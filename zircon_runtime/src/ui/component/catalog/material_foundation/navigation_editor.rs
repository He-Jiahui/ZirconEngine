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
    editor_panel_component(
        "ContextMenu",
        "Context Menu",
        UiComponentCategory::Input,
        "context-menu",
    )
    .slot(multi_slot("items"))
    .events([
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
