use super::family::TemplateComponentFamily;

pub(in crate::ui::retained_host::host_contract) fn role_family(
    role: &str,
) -> Option<TemplateComponentFamily> {
    match role {
        "button" | "toggle-button" => Some(TemplateComponentFamily::Button),
        "icon-button" => Some(TemplateComponentFamily::IconButton),
        "input-field" | "text-field" | "number-field" | "color-field" | "vector-field" => {
            Some(TemplateComponentFamily::TextInput)
        }
        "key-selector" => Some(TemplateComponentFamily::KeySelector),
        "range-field" | "slider" => Some(TemplateComponentFamily::Slider),
        "checkbox" => Some(TemplateComponentFamily::Checkbox),
        "radio" => Some(TemplateComponentFamily::Radio),
        "toggle" | "switch" => Some(TemplateComponentFamily::Toggle),
        "dropdown" | "combo-box" | "select" | "search-select" => {
            Some(TemplateComponentFamily::Dropdown)
        }
        "tab" => Some(TemplateComponentFamily::Tab),
        "segmented-control" => Some(TemplateComponentFamily::SegmentedControl),
        "list" | "virtual-list" => Some(TemplateComponentFamily::List),
        "list-row" | "list-item" => Some(TemplateComponentFamily::ListRow),
        "tree-row" | "tree-item" => Some(TemplateComponentFamily::TreeRow),
        "table" | "editable-table" => Some(TemplateComponentFamily::Table),
        "table-row" => Some(TemplateComponentFamily::TableRow),
        "menu"
        | "context-menu"
        | "context-action-menu"
        | "dropdown-popup"
        | "popover"
        | "popper" => Some(TemplateComponentFamily::Popup),
        "tooltip" => Some(TemplateComponentFamily::Tooltip),
        "alert" | "toast" | "snackbar" => Some(TemplateComponentFamily::Alert),
        "container" | "panel" | "paper" | "card" | "toolbar" | "property-grid"
        | "inspector-section" => Some(TemplateComponentFamily::Container),
        "drawer" => Some(TemplateComponentFamily::Drawer),
        "window" | "window-view" => Some(TemplateComponentFamily::Window),
        _ => None,
    }
}

pub(in crate::ui::retained_host::host_contract) fn host_role_family(
    role: &str,
) -> Option<TemplateComponentFamily> {
    match role {
        "Button" => Some(TemplateComponentFamily::Button),
        "IconButton" => Some(TemplateComponentFamily::IconButton),
        "InputField" | "TextField" | "SearchField" => Some(TemplateComponentFamily::TextInput),
        "KeySelector" => Some(TemplateComponentFamily::KeySelector),
        "Slider" | "RangeField" => Some(TemplateComponentFamily::Slider),
        "Checkbox" => Some(TemplateComponentFamily::Checkbox),
        "Radio" => Some(TemplateComponentFamily::Radio),
        "Toggle" | "Switch" => Some(TemplateComponentFamily::Toggle),
        "Dropdown" | "ComboBox" | "Select" => Some(TemplateComponentFamily::Dropdown),
        "Tab" => Some(TemplateComponentFamily::Tab),
        "SegmentedControl" => Some(TemplateComponentFamily::SegmentedControl),
        "List" | "VirtualList" => Some(TemplateComponentFamily::List),
        "ListRow" => Some(TemplateComponentFamily::ListRow),
        "TreeRow" => Some(TemplateComponentFamily::TreeRow),
        "Table" => Some(TemplateComponentFamily::Table),
        "TableRow" => Some(TemplateComponentFamily::TableRow),
        "Popup" | "Menu" | "ContextActionMenu" => Some(TemplateComponentFamily::Popup),
        "Tooltip" => Some(TemplateComponentFamily::Tooltip),
        "Alert" | "Toast" | "Snackbar" => Some(TemplateComponentFamily::Alert),
        "Container" | "Panel" | "Paper" | "Card" | "Toolbar" => {
            Some(TemplateComponentFamily::Container)
        }
        "Drawer" => Some(TemplateComponentFamily::Drawer),
        "Window" | "WindowView" => Some(TemplateComponentFamily::Window),
        _ => None,
    }
}
