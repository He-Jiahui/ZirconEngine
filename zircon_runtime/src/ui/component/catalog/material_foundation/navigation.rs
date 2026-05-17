use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        composite(
            "Breadcrumbs",
            "Breadcrumbs",
            UiComponentCategory::Container,
            "breadcrumbs",
        )
        .slot(UiSlotSchema::new("items").multiple(true))
        .event(UiComponentEventKind::SelectOption),
        composite(
            "BottomNavigation",
            "Bottom Navigation",
            UiComponentCategory::Container,
            "bottom-navigation",
        )
        .with_prop(value_text_prop())
        .slot(UiSlotSchema::new("actions").multiple(true))
        .event(UiComponentEventKind::SelectOption),
        shell("Drawer", "Drawer", "drawer")
            .with_prop(enum_prop("slot", "left_top"))
            .with_prop(enum_prop("mode", "pinned"))
            .with_prop(string_prop("active_view"))
            .slot(UiSlotSchema::new("tabs").multiple(true))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::SelectOption),
        primitive("Link", "Link", UiComponentCategory::Input, "link")
            .with_prop(text_prop())
            .with_prop(string_prop("href"))
            .event(UiComponentEventKind::Commit),
        composite("Menu", "Menu", UiComponentCategory::Input, "menu")
            .slot(UiSlotSchema::new("items").multiple(true))
            .event(UiComponentEventKind::Commit),
        composite("Menubar", "Menubar", UiComponentCategory::Input, "menubar")
            .slot(UiSlotSchema::new("items").multiple(true))
            .events([
                UiComponentEventKind::OpenPopup,
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ClosePopup,
            ]),
        composite(
            "Pagination",
            "Pagination",
            UiComponentCategory::Input,
            "pagination",
        )
        .with_prop(int_prop("page", 1))
        .with_prop(int_prop("page_count", 10))
        .event(UiComponentEventKind::SetPage),
        composite(
            "Stepper",
            "Stepper",
            UiComponentCategory::Container,
            "stepper",
        )
        .with_prop(int_prop("active_step", 0))
        .slot(UiSlotSchema::new("steps").multiple(true))
        .event(UiComponentEventKind::SelectOption),
        composite("Tabs", "Tabs", UiComponentCategory::Container, "tabs")
            .with_prop(value_text_prop())
            .slot(UiSlotSchema::new("tabs").multiple(true))
            .slot(UiSlotSchema::new("panels").multiple(true))
            .event(UiComponentEventKind::ValueChanged),
        composite(
            "TransferList",
            "Transfer List",
            UiComponentCategory::Collection,
            "transfer-list",
        )
        .with_prop(array_prop("source_items"))
        .with_prop(array_prop("target_items"))
        .events([
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::MoveElement,
        ]),
        editor_panel_component(
            "ContextMenu",
            "Context Menu",
            UiComponentCategory::Input,
            "context-menu",
        )
        .slot(UiSlotSchema::new("items").multiple(true))
        .events([
            UiComponentEventKind::OpenPopupAt,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::Commit,
        ]),
        editor_panel_component(
            "PaneToolbar",
            "Pane Toolbar",
            UiComponentCategory::Container,
            "pane-toolbar",
        )
        .slot(UiSlotSchema::new("actions").multiple(true))
        .event(UiComponentEventKind::Commit),
        editor_panel_component(
            "FilterBar",
            "Filter Bar",
            UiComponentCategory::Input,
            "filter-bar",
        )
        .with_prop(string_prop("query"))
        .with_prop(enum_prop("severity", "all"))
        .slot(UiSlotSchema::new("filters").multiple(true))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::SelectOption,
        ]),
        editor_panel_component(
            "SeverityChips",
            "Severity Chips",
            UiComponentCategory::Selection,
            "severity-chips",
        )
        .with_prop(enum_prop("selected_severity", "all"))
        .event(UiComponentEventKind::SelectOption),
        shell("ViewTab", "View Tab", "view-tab")
            .with_prop(required_string_prop("view_id"))
            .with_prop(text_prop())
            .events([
                UiComponentEventKind::Commit,
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::EndDrag,
            ]),
        shell("TabStack", "Tab Stack", "tab-stack")
            .with_prop(string_prop("active_tab"))
            .slot(UiSlotSchema::new("tabs").multiple(true))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::SelectOption),
    ]
}
