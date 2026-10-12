use zircon_runtime_interface::ui::component::UiComponentDescriptor;

use crate::ui::component_registry::retained_component_registry;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ComponentContractTokens {
    pub category: &'static str,
    pub layout_role: &'static str,
}

pub(super) fn descriptor_for_component(
    component_id: &str,
) -> Option<&'static UiComponentDescriptor> {
    retained_component_registry().descriptor(component_id)
}

pub(super) fn tokens_for_component_role(
    component_id: &str,
    component_role: &str,
) -> ComponentContractTokens {
    descriptor_for_component(component_id)
        .map(|descriptor| ComponentContractTokens {
            category: descriptor.category.as_str(),
            layout_role: descriptor.layout_role.as_str(),
        })
        .unwrap_or_else(|| fallback_tokens_for_component_role(component_id, component_role))
}

fn fallback_tokens_for_component_role(
    component_id: &str,
    component_role: &str,
) -> ComponentContractTokens {
    ComponentContractTokens {
        category: fallback_category_for_component_role(component_role),
        layout_role: fallback_layout_role_for_component_role(component_id, component_role),
    }
}

fn fallback_category_for_component_role(component_role: &str) -> &'static str {
    match component_role {
        "text" | "label" | "image" | "icon" | "svg" | "svg-icon" | "divider" => "visual",
        "number-field" | "range-field" | "slider" | "color-field" | "vector-field" => "numeric",
        "dropdown"
        | "select"
        | "search-select"
        | "segmented-control"
        | "tab"
        | "menu"
        | "context-menu"
        | "dropdown-popup"
        | "context-action-menu" => "selection",
        "reference-field" | "asset-field" | "scene-reference-field" => "reference",
        "table" | "table-row" | "list" | "list-item" | "tree-row" | "virtual-list" => "collection",
        "container" | "panel" | "paper" | "card" | "toolbar" | "drawer" | "inspector-section"
        | "property-grid" => "container",
        "alert" | "tooltip" | "toast" | "snackbar" | "progress" | "badge" | "skeleton" => {
            "feedback"
        }
        "button" | "icon-button" | "toggle-button" | "input-field" | "text-field"
        | "command-palette" | "checkbox" | "radio" | "switch" | "toggle" => "input",
        _ => "",
    }
}

fn fallback_layout_role_for_component_role(
    component_id: &str,
    component_role: &str,
) -> &'static str {
    match component_id {
        "HorizontalBox" | "HorizontalGroup" | "VerticalBox" | "VerticalGroup" | "Container"
        | "Panel" | "Toolbar" | "Drawer" | "Card" | "Paper" => "flex",
        "Grid" | "GridBox" | "Table" => "grid",
        "Popup" | "Popover" | "Popper" | "Tooltip" | "Menu" | "ContextActionMenu"
        | "CommandPalette" => "popup",
        "Canvas" => "canvas",
        "VirtualList" => "virtual-list",
        _ => match component_role {
            "container" | "panel" | "paper" | "card" | "toolbar" | "drawer" => "flex",
            "table" => "grid",
            "tooltip"
            | "popover"
            | "popper"
            | "menu"
            | "command-palette"
            | "context-menu"
            | "context-action-menu"
            | "dropdown-popup" => "popup",
            "virtual-list" => "virtual-list",
            _ => "leaf",
        },
    }
}
