use super::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum TemplateComponentFamily {
    Button,
    IconButton,
    TextInput,
    Slider,
    Checkbox,
    Radio,
    Toggle,
    Dropdown,
    Tab,
    SegmentedControl,
    List,
    ListRow,
    TreeRow,
    Table,
    TableRow,
    Popup,
    Tooltip,
    Alert,
    Container,
    Drawer,
    Window,
}

impl TemplateComponentFamily {
    pub(in crate::ui::retained_host::host_contract) fn as_str(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::IconButton => "icon-button",
            Self::TextInput => "text-input",
            Self::Slider => "slider",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Toggle => "toggle",
            Self::Dropdown => "dropdown",
            Self::Tab => "tab",
            Self::SegmentedControl => "segmented-control",
            Self::List => "list",
            Self::ListRow => "list-row",
            Self::TreeRow => "tree-row",
            Self::Table => "table",
            Self::TableRow => "table-row",
            Self::Popup => "popup",
            Self::Tooltip => "tooltip",
            Self::Alert => "alert",
            Self::Container => "container",
            Self::Drawer => "drawer",
            Self::Window => "window",
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn template_component_family(
    node: &TemplatePaneNodeData,
) -> Option<TemplateComponentFamily> {
    let role = node.component_role.as_str();
    let category = node.component_category.as_str();
    let layout_role = node.component_layout_role.as_str();
    let host_role = node.role.as_str();
    let control_id = node.control_id.as_str();

    role_family(role)
        .or_else(|| host_role_family(host_role))
        .or_else(|| category_layout_family(category, layout_role))
        .or_else(|| workbench_control_family(control_id))
}

pub(in crate::ui::retained_host::host_contract) fn is_component_family(
    node: &TemplatePaneNodeData,
    family: TemplateComponentFamily,
) -> bool {
    template_component_family(node) == Some(family)
}

pub(in crate::ui::retained_host::host_contract) fn is_any_component_family(
    node: &TemplatePaneNodeData,
    families: &[TemplateComponentFamily],
) -> bool {
    template_component_family(node)
        .map(|family| families.contains(&family))
        .unwrap_or(false)
}

pub(in crate::ui::retained_host::host_contract) fn uses_workbench_visual_language(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str().starts_with("Workbench")
        || node.component_variant.as_str().contains("workbench")
        || node.surface_variant.as_str().contains("workbench")
        || node.button_variant.as_str().contains("workbench")
}

fn role_family(role: &str) -> Option<TemplateComponentFamily> {
    match role {
        "button" | "toggle-button" => Some(TemplateComponentFamily::Button),
        "icon-button" => Some(TemplateComponentFamily::IconButton),
        "input-field" | "text-field" | "number-field" | "color-field" | "vector-field" => {
            Some(TemplateComponentFamily::TextInput)
        }
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

fn host_role_family(role: &str) -> Option<TemplateComponentFamily> {
    match role {
        "Button" => Some(TemplateComponentFamily::Button),
        "IconButton" => Some(TemplateComponentFamily::IconButton),
        "InputField" | "TextField" => Some(TemplateComponentFamily::TextInput),
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

fn category_layout_family(category: &str, layout_role: &str) -> Option<TemplateComponentFamily> {
    match (category, layout_role) {
        ("collection", "grid") => Some(TemplateComponentFamily::Table),
        ("collection", "virtual-list") => Some(TemplateComponentFamily::List),
        ("container", "editor-dock") => Some(TemplateComponentFamily::Window),
        ("container", "flex" | "grid") => Some(TemplateComponentFamily::Container),
        ("selection", "popup") => Some(TemplateComponentFamily::Popup),
        ("feedback", "popup") => Some(TemplateComponentFamily::Tooltip),
        _ => None,
    }
}

fn workbench_control_family(control_id: &str) -> Option<TemplateComponentFamily> {
    if control_id.starts_with("WorkbenchMini")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchRail")
        || control_id.contains("IconButton")
    {
        Some(TemplateComponentFamily::IconButton)
    } else if control_id.starts_with("WorkbenchCheckbox") {
        Some(TemplateComponentFamily::Checkbox)
    } else if control_id.starts_with("WorkbenchRadio") {
        Some(TemplateComponentFamily::Radio)
    } else if control_id.starts_with("WorkbenchToggle") {
        Some(TemplateComponentFamily::Toggle)
    } else if control_id.starts_with("WorkbenchDrawerTab")
        || control_id.starts_with("WorkbenchLabsTab")
    {
        Some(TemplateComponentFamily::Tab)
    } else if control_id.contains("Segmented") {
        Some(TemplateComponentFamily::SegmentedControl)
    } else if control_id.starts_with("WorkbenchInputSlider")
        || control_id.starts_with("WorkbenchInputRangeSlider")
        || control_id.starts_with("WorkbenchInputStepsSlider")
        || control_id.starts_with("WorkbenchSlider")
    {
        Some(TemplateComponentFamily::Slider)
    } else if control_id == "WorkbenchInputDropdown" || control_id.starts_with("WorkbenchDropdown")
    {
        Some(TemplateComponentFamily::Dropdown)
    } else if control_id.starts_with("WorkbenchInput") || control_id.starts_with("WorkbenchField") {
        Some(TemplateComponentFamily::TextInput)
    } else if control_id.starts_with("WorkbenchList") {
        Some(TemplateComponentFamily::ListRow)
    } else if control_id.starts_with("WorkbenchSceneVirtualItem")
        || (control_id.starts_with("WorkbenchScene") && control_id.ends_with("Item"))
        || control_id.starts_with("WorkbenchEffectAsset")
        || control_id.starts_with("WorkbenchEffectHierarchy")
    {
        Some(TemplateComponentFamily::TreeRow)
    } else if control_id.starts_with("WorkbenchTable")
        || control_id.starts_with("WorkbenchEffectModifier")
    {
        Some(TemplateComponentFamily::TableRow)
    } else if control_id.ends_with("Button") || control_id.contains("Button") {
        Some(TemplateComponentFamily::Button)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_family_prefers_declared_component_role() {
        let node = node_with_contract("AnyControl", "input", "button", "leaf");

        assert_eq!(
            template_component_family(&node),
            Some(TemplateComponentFamily::Button)
        );
    }

    #[test]
    fn component_family_uses_category_and_layout_for_collections() {
        let grid = node_with_contract("AnyTable", "collection", "", "grid");
        let list = node_with_contract("AnyList", "collection", "", "virtual-list");

        assert_eq!(
            template_component_family(&grid),
            Some(TemplateComponentFamily::Table)
        );
        assert_eq!(
            template_component_family(&list),
            Some(TemplateComponentFamily::List)
        );
    }

    #[test]
    fn workbench_visual_language_can_be_declared_without_control_prefix() {
        let mut node = node_with_contract("Primary", "input", "button", "leaf");
        node.component_variant = "workbench-button".into();

        assert!(uses_workbench_visual_language(&node));
        assert!(is_component_family(&node, TemplateComponentFamily::Button));
    }

    #[test]
    fn range_field_is_a_slider_family() {
        let range = node_with_contract("AnyRange", "input", "range-field", "leaf");
        let by_id = node_with_contract("WorkbenchInputSlider", "", "", "");

        assert_eq!(
            template_component_family(&range),
            Some(TemplateComponentFamily::Slider)
        );
        assert_eq!(
            template_component_family(&by_id),
            Some(TemplateComponentFamily::Slider)
        );
    }

    fn node_with_contract(
        control_id: &str,
        category: &str,
        role: &str,
        layout_role: &str,
    ) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            component_category: category.into(),
            component_role: role.into(),
            component_layout_role: layout_role.into(),
            ..TemplatePaneNodeData::default()
        }
    }
}
