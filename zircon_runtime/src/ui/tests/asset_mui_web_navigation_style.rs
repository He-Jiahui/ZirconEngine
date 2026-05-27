use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const NAV_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_navigation_style"
version = 1
display_name = "MUI Web Navigation Style"

[[stylesheets]]
id = "mui_web_navigation"

[[stylesheets.rules]]
selector = ".MuiLink-underlineHover.MuiLink-button.Mui-focusVisible"
set = { self = { text_tone = "info" } }

[[stylesheets.rules]]
selector = ".MuiBottomNavigationAction-iconOnly"
set = { self = { surface_variant = "nav-action" } }

[[stylesheets.rules]]
selector = ".MuiBottomNavigationAction-label.MuiBottomNavigationAction-iconOnly.label-extra"
set = { self = { text_tone = "muted" } }

[[stylesheets.rules]]
selector = ".MuiMenuItem-dense.MuiMenuItem-gutters.MuiMenuItem-divider.Mui-selected"
set = { self = { validation_level = "success" } }

[[stylesheets.rules]]
selector = ".MuiPagination-outlined"
set = { self = { surface_variant = "pagination" } }

[[stylesheets.rules]]
selector = ".MuiPagination-ul.nav-ul-extra"
set = { self = { text_align = "center" } }

[[stylesheets.rules]]
selector = ".MuiPaginationItem-outlined.MuiPaginationItem-rounded.MuiPaginationItem-colorPrimary.MuiPaginationItem-page.Mui-selected"
set = { self = { validation_level = "info" } }

[[stylesheets.rules]]
selector = ".MuiStepper-vertical.MuiStepper-nonLinear.MuiStepper-alternativeLabel"
set = { self = { surface_variant = "stepper" } }

[[stylesheets.rules]]
selector = ".MuiStepConnector-vertical.MuiStepConnector-alternativeLabel.Mui-active"
set = { self = { border_width = 2.0 } }

[[stylesheets.rules]]
selector = ".MuiStepLabel-label.Mui-completed.MuiStepLabel-alternativeLabel"
set = { self = { text_tone = "success" } }

[[stylesheets.rules]]
selector = ".MuiTabs-vertical"
set = { self = { surface_variant = "tabs" } }

[[stylesheets.rules]]
selector = ".MuiTabs-scroller.MuiTabs-scrollableY.MuiTabs-hideScrollbar"
set = { self = { overflow = "scroll" } }

[[stylesheets.rules]]
selector = ".MuiTabs-list.MuiTabs-vertical.MuiTabs-centered"
set = { self = { text_align = "center" } }

[[stylesheets.rules]]
selector = ".MuiTab-labelIcon.MuiTab-textColorSecondary.MuiTab-fullWidth.MuiTab-wrapped.Mui-selected"
set = { self = { validation_level = "warning" } }

[[stylesheets.rules]]
selector = ".MuiTab-icon.tab-icon-extra"
set = { self = { text_tone = "warning" } }

[[stylesheets.rules]]
selector = ".MuiTransferList-root.MuiTransferList-hasSourceItems.MuiTransferList-hasTargetItems.MuiTransferList-hasSelectedItems.MuiTransferList-hasDisabledItems.MuiTransferList-hasDisabledActions"
set = { self = { surface_variant = "transfer-list" } }

[[stylesheets.rules]]
selector = ".MuiTransferList-source.MuiTransferList-sourcePopulated.MuiTransferList-sourceSelected.source-extra"
set = { self = { text_tone = "info" } }

[[stylesheets.rules]]
selector = ".MuiTransferList-target.MuiTransferList-targetPopulated.MuiTransferList-targetSelected.target-extra"
set = { self = { text_tone = "success" } }

[[stylesheets.rules]]
selector = ".MuiTransferList-actions.MuiTransferList-actionsDisabled.actions-extra"
set = { self = { validation_level = "warning" } }
"##;

const NAV_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_navigation_style_layout"
version = 1
display_name = "MUI Web Navigation Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_navigation_style.ui"]

[root]
node_id = "navigation_root"
kind = "native"
type = "VerticalBox"
control_id = "NavigationRoot"

[[root.children]]
[root.children.node]
node_id = "navigation_link"
kind = "native"
type = "Link"
control_id = "NavigationLink"
props = { text = "Docs", component = "button", underline = "hover", focusVisible = true }

[[root.children]]
[root.children.node]
node_id = "bottom_action"
kind = "native"
type = "BottomNavigationAction"
control_id = "BottomAction"
props = { label = "Home", showLabel = false, slotProps = { label = { className = "label-extra" } } }

[[root.children.node.children]]
mount = "label"
[root.children.node.children.node]
node_id = "bottom_action_label"
kind = "native"
type = "Label"
control_id = "BottomActionLabel"
props = { text = "Home" }

[[root.children]]
[root.children.node]
node_id = "menu_item"
kind = "native"
type = "MenuItem"
control_id = "MenuItemDense"
props = { text = "Open", dense = true, divider = true, selected = true }

[[root.children]]
[root.children.node]
node_id = "pagination"
kind = "native"
type = "Pagination"
control_id = "PaginationRoot"
props = { variant = "outlined", slotProps = { ul = { className = "nav-ul-extra" } } }

[[root.children.node.children]]
mount = "ul"
[root.children.node.children.node]
node_id = "pagination_ul"
kind = "native"
type = "Label"
control_id = "PaginationUl"
props = { text = "Pages" }

[[root.children]]
[root.children.node]
node_id = "pagination_item"
kind = "native"
type = "PaginationItem"
control_id = "PaginationItemPage"
props = { type = "page", variant = "outlined", shape = "rounded", color = "primary", selected = true, size = "large" }

[[root.children]]
[root.children.node]
node_id = "stepper"
kind = "native"
type = "Stepper"
control_id = "StepperRoot"
props = { orientation = "vertical", nonLinear = true, alternativeLabel = true }

[[root.children]]
[root.children.node]
node_id = "step_connector"
kind = "native"
type = "StepConnector"
control_id = "StepConnectorRoot"
props = { orientation = "vertical", alternativeLabel = true, active = true }

[[root.children.node.children]]
mount = "line"
[root.children.node.children.node]
node_id = "step_connector_line"
kind = "native"
type = "Label"
control_id = "StepConnectorLine"
props = { text = "Line" }

[[root.children]]
[root.children.node]
node_id = "step_label"
kind = "native"
type = "StepLabel"
control_id = "StepLabelRoot"
props = { completed = true, alternativeLabel = true, orientation = "vertical" }

[[root.children.node.children]]
mount = "label"
[root.children.node.children.node]
node_id = "step_label_text"
kind = "native"
type = "Label"
control_id = "StepLabelText"
props = { text = "Done" }

[[root.children]]
[root.children.node]
node_id = "tabs_scrollable"
kind = "native"
type = "Tabs"
control_id = "TabsScrollable"
props = { variant = "scrollable", orientation = "vertical" }

[[root.children.node.children]]
mount = "scroller"
[root.children.node.children.node]
node_id = "tabs_scroller"
kind = "native"
type = "Label"
control_id = "TabsScroller"
props = { text = "Scroller" }

[[root.children]]
[root.children.node]
node_id = "tabs_centered"
kind = "native"
type = "Tabs"
control_id = "TabsCentered"
props = { orientation = "vertical", centered = true }

[[root.children.node.children]]
mount = "list"
[root.children.node.children.node]
node_id = "tabs_list"
kind = "native"
type = "Label"
control_id = "TabsList"
props = { text = "List" }

[[root.children]]
[root.children.node]
node_id = "tab"
kind = "native"
type = "Tab"
control_id = "TabRoot"
props = { label = "Scene", textColor = "secondary", fullWidth = true, wrapped = true, selected = true, slotProps = { icon = { className = "tab-icon-extra" } } }

[[root.children.node.children]]
mount = "icon"
[root.children.node.children.node]
node_id = "tab_icon"
kind = "native"
type = "Label"
control_id = "TabIcon"
props = { text = "Icon" }

[[root.children]]
[root.children.node]
node_id = "transfer_list"
kind = "native"
type = "TransferList"
control_id = "TransferListRoot"
props = { sourceItems = ["scene", "materials"], targetItems = ["export"], sourceSelectedItems = ["materials"], targetSelectedItems = ["export"], disabledItems = ["scene"], disabledActions = ["move_all_left"], slotProps = { source = { className = "source-extra" }, target = { className = "target-extra" }, actions = { className = "actions-extra" } } }

[[root.children.node.children]]
mount = "source"
[root.children.node.children.node]
node_id = "transfer_source"
kind = "native"
type = "List"
control_id = "TransferSource"
props = { items = ["scene", "materials"] }

[[root.children.node.children]]
mount = "target"
[root.children.node.children.node]
node_id = "transfer_target"
kind = "native"
type = "List"
control_id = "TransferTarget"
props = { items = ["export"] }

[[root.children.node.children]]
mount = "actions"
[root.children.node.children.node]
node_id = "transfer_actions"
kind = "native"
type = "Button"
control_id = "TransferActions"
props = { text = "Move" }
"##;

#[test]
fn mui_web_navigation_utility_classes_match_local_material_contracts() {
    let style = UiAssetLoader::load_toml_str(NAV_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(NAV_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_navigation_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let link = find_node(root, "NavigationLink");
    assert_eq!(str_attr(link, "text_tone"), Some("info"));
    assert_classes(
        link,
        &[
            "MuiLink-root",
            "MuiLink-underlineHover",
            "MuiLink-button",
            "Mui-focusVisible",
        ],
    );

    let bottom_action = find_node(root, "BottomAction");
    assert_eq!(
        str_attr(bottom_action, "surface_variant"),
        Some("nav-action")
    );
    assert_classes(
        bottom_action,
        &[
            "MuiBottomNavigationAction-root",
            "MuiBottomNavigationAction-iconOnly",
        ],
    );
    let bottom_label = find_node(root, "BottomActionLabel");
    assert_eq!(str_attr(bottom_label, "text_tone"), Some("muted"));
    assert_classes(
        bottom_label,
        &[
            "MuiBottomNavigationAction-label",
            "MuiBottomNavigationAction-iconOnly",
            "label-extra",
        ],
    );

    let menu_item = find_node(root, "MenuItemDense");
    assert_eq!(str_attr(menu_item, "validation_level"), Some("success"));
    assert_classes(
        menu_item,
        &[
            "MuiMenuItem-root",
            "MuiMenuItem-dense",
            "MuiMenuItem-gutters",
            "MuiMenuItem-divider",
            "Mui-selected",
        ],
    );

    let pagination = find_node(root, "PaginationRoot");
    assert_eq!(str_attr(pagination, "surface_variant"), Some("pagination"));
    assert_classes(
        pagination,
        &["MuiPagination-root", "MuiPagination-outlined"],
    );
    let pagination_ul = find_node(root, "PaginationUl");
    assert_eq!(str_attr(pagination_ul, "text_align"), Some("center"));
    assert_classes(pagination_ul, &["MuiPagination-ul", "nav-ul-extra"]);

    let pagination_item = find_node(root, "PaginationItemPage");
    assert_eq!(str_attr(pagination_item, "validation_level"), Some("info"));
    assert_classes(
        pagination_item,
        &[
            "MuiPaginationItem-root",
            "MuiPaginationItem-sizeLarge",
            "MuiPaginationItem-outlined",
            "MuiPaginationItem-rounded",
            "MuiPaginationItem-colorPrimary",
            "MuiPaginationItem-page",
            "Mui-selected",
        ],
    );

    let stepper = find_node(root, "StepperRoot");
    assert_eq!(str_attr(stepper, "surface_variant"), Some("stepper"));
    assert_classes(
        stepper,
        &[
            "MuiStepper-root",
            "MuiStepper-vertical",
            "MuiStepper-nonLinear",
            "MuiStepper-alternativeLabel",
        ],
    );

    let step_connector = find_node(root, "StepConnectorRoot");
    assert_eq!(float_attr(step_connector, "border_width"), Some(2.0));
    assert_classes(
        step_connector,
        &[
            "MuiStepConnector-root",
            "MuiStepConnector-vertical",
            "MuiStepConnector-alternativeLabel",
            "Mui-active",
        ],
    );
    assert_classes(
        find_node(root, "StepConnectorLine"),
        &["MuiStepConnector-line", "MuiStepConnector-lineVertical"],
    );

    let step_label_text = find_node(root, "StepLabelText");
    assert_eq!(str_attr(step_label_text, "text_tone"), Some("success"));
    assert_classes(
        step_label_text,
        &[
            "MuiStepLabel-label",
            "Mui-completed",
            "MuiStepLabel-alternativeLabel",
        ],
    );

    let tabs_scrollable = find_node(root, "TabsScrollable");
    assert_eq!(str_attr(tabs_scrollable, "surface_variant"), Some("tabs"));
    assert_classes(tabs_scrollable, &["MuiTabs-root", "MuiTabs-vertical"]);
    let tabs_scroller = find_node(root, "TabsScroller");
    assert_eq!(str_attr(tabs_scroller, "overflow"), Some("scroll"));
    assert_classes(
        tabs_scroller,
        &[
            "MuiTabs-scroller",
            "MuiTabs-scrollableY",
            "MuiTabs-hideScrollbar",
        ],
    );

    let tabs_list = find_node(root, "TabsList");
    assert_eq!(str_attr(tabs_list, "text_align"), Some("center"));
    assert_classes(
        tabs_list,
        &["MuiTabs-list", "MuiTabs-vertical", "MuiTabs-centered"],
    );

    let tab = find_node(root, "TabRoot");
    assert_eq!(str_attr(tab, "validation_level"), Some("warning"));
    assert_classes(
        tab,
        &[
            "MuiTab-root",
            "MuiTab-labelIcon",
            "MuiTab-textColorSecondary",
            "MuiTab-fullWidth",
            "MuiTab-wrapped",
            "Mui-selected",
        ],
    );
    let tab_icon = find_node(root, "TabIcon");
    assert_eq!(str_attr(tab_icon, "text_tone"), Some("warning"));
    assert_classes(tab_icon, &["MuiTab-icon", "tab-icon-extra"]);

    let transfer_list = find_node(root, "TransferListRoot");
    assert_eq!(
        str_attr(transfer_list, "surface_variant"),
        Some("transfer-list")
    );
    assert_classes(
        transfer_list,
        &[
            "MuiTransferList-root",
            "MuiTransferList-hasSourceItems",
            "MuiTransferList-hasTargetItems",
            "MuiTransferList-hasSelectedItems",
            "MuiTransferList-hasDisabledItems",
            "MuiTransferList-hasDisabledActions",
        ],
    );

    let transfer_source = find_node(root, "TransferSource");
    assert_eq!(str_attr(transfer_source, "text_tone"), Some("info"));
    assert_classes(
        transfer_source,
        &[
            "MuiTransferList-source",
            "MuiTransferList-sourcePopulated",
            "MuiTransferList-sourceSelected",
            "source-extra",
        ],
    );

    let transfer_target = find_node(root, "TransferTarget");
    assert_eq!(str_attr(transfer_target, "text_tone"), Some("success"));
    assert_classes(
        transfer_target,
        &[
            "MuiTransferList-target",
            "MuiTransferList-targetPopulated",
            "MuiTransferList-targetSelected",
            "target-extra",
        ],
    );

    let transfer_actions = find_node(root, "TransferActions");
    assert_eq!(
        str_attr(transfer_actions, "validation_level"),
        Some("warning")
    );
    assert_classes(
        transfer_actions,
        &[
            "MuiTransferList-actions",
            "MuiTransferList-actionsDisabled",
            "actions-extra",
        ],
    );
}

fn find_node<'a>(node: &'a UiTemplateNode, control_id: &str) -> &'a UiTemplateNode {
    if node.control_id.as_deref() == Some(control_id) {
        return node;
    }
    for child in &node.children {
        if let Some(found) = find_node_opt(child, control_id) {
            return found;
        }
    }
    panic!("missing node `{control_id}`");
}

fn find_node_opt<'a>(node: &'a UiTemplateNode, control_id: &str) -> Option<&'a UiTemplateNode> {
    if node.control_id.as_deref() == Some(control_id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node_opt(child, control_id))
}

fn str_attr<'a>(node: &'a UiTemplateNode, name: &str) -> Option<&'a str> {
    node.attributes.get(name).and_then(Value::as_str)
}

fn float_attr(node: &UiTemplateNode, name: &str) -> Option<f64> {
    node.attributes.get(name).and_then(Value::as_float)
}

fn assert_classes(node: &UiTemplateNode, expected: &[&str]) {
    for class_name in expected {
        assert!(
            node.classes.iter().any(|value| value == class_name),
            "missing {class_name} in {:?}",
            node.classes
        );
    }
}
