use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const SURFACE_CHILD_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_surface_child_style"
version = 1
display_name = "MUI Web Surface Child Style"

[[stylesheets]]
id = "mui_web_surface_child"

[[stylesheets.rules]]
selector = ".MuiAccordionSummary-root.MuiAccordionSummary-gutters.MuiAccordionSummary-expanded.Mui-expanded.MuiAccordionSummary-focusVisible.Mui-focusVisible.summary-focus-extra"
set = { self = { surface_variant = "accordion-summary-active" } }

[[stylesheets.rules]]
selector = ".MuiAccordionSummary-content.Mui-expanded.summary-content-extra"
set = { self = { text_tone = "accordion-content" } }

[[stylesheets.rules]]
selector = ".MuiAccordionSummary-expandIconWrapper.Mui-expanded.summary-icon-extra"
set = { self = { text_tone = "accordion-icon" } }

[[stylesheets.rules]]
selector = ".MuiAccordionActions-spacing"
set = { self = { surface_variant = "accordion-actions" } }

[[stylesheets.rules]]
selector = ".MuiDialogActions-spacing"
set = { self = { surface_variant = "dialog-actions" } }

[[stylesheets.rules]]
selector = ".MuiDialogContent-dividers"
set = { self = { surface_variant = "dialog-content-dividers" } }

[[stylesheets.rules]]
selector = ".MuiMobileStepper-positionTop"
set = { self = { surface_variant = "mobile-stepper-top" } }

[[stylesheets.rules]]
selector = ".MuiMobileStepper-dot.MuiMobileStepper-dotActive.dot-active-extra"
set = { self = { validation_level = "active-dot" } }

[[stylesheets.rules]]
selector = ".MuiMobileStepper-progress.progress-extra"
set = { self = { surface_variant = "mobile-progress" } }

[[stylesheets.rules]]
selector = ".MuiSpeedDialAction-staticTooltip.MuiSpeedDialAction-staticTooltipClosed.MuiSpeedDialAction-tooltipPlacementRight"
set = { self = { surface_variant = "speed-action-tooltip" } }

[[stylesheets.rules]]
selector = ".MuiSpeedDialAction-fab.MuiSpeedDialAction-fabClosed.fab-extra"
set = { self = { text_tone = "fab-closed" } }

[[stylesheets.rules]]
selector = ".MuiSpeedDialAction-staticTooltipLabel.tooltip-label-extra"
set = { self = { text_tone = "tooltip-label" } }

[[stylesheets.rules]]
selector = ".MuiSpeedDialIcon-icon.MuiSpeedDialIcon-iconOpen.MuiSpeedDialIcon-iconWithOpenIconOpen"
set = { self = { text_tone = "speed-icon-open" } }

[[stylesheets.rules]]
selector = ".MuiSpeedDialIcon-openIcon.MuiSpeedDialIcon-openIconOpen"
set = { self = { text_tone = "speed-open-icon" } }

[[stylesheets.rules]]
selector = ".MuiDrawer-root.MuiDrawer-anchorRight.MuiDrawer-modal"
set = { self = { surface_variant = "swipeable-drawer-root" } }

[[stylesheets.rules]]
selector = ".MuiDrawer-paper.drawer-paper-extra"
set = { self = { surface_variant = "swipeable-drawer-paper" } }

[[stylesheets.rules]]
selector = ".PrivateSwipeArea-root.PrivateSwipeArea-anchorRight.swipe-area-extra"
set = { self = { surface_variant = "swipe-area" } }

[[stylesheets.rules]]
selector = ".MuiTabScrollButton-root.MuiTabScrollButton-vertical.MuiTabScrollButton-disabled.Mui-disabled"
set = { self = { validation_level = "tab-scroll-disabled" } }
"##;

const SURFACE_CHILD_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_surface_child_style_layout"
version = 1
display_name = "MUI Web Surface Child Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_surface_child_style.ui"]

[root]
node_id = "surface_child_root"
kind = "native"
type = "VerticalBox"
control_id = "SurfaceChildRoot"

[[root.children]]
[root.children.node]
node_id = "accordion_summary"
kind = "native"
type = "AccordionSummary"
control_id = "AccordionSummaryRoot"
props = { text = "Details", expanded = true, focusVisible = true, focusVisibleClassName = "summary-focus-extra", slotProps = { content = { className = "summary-content-extra" }, expandIconWrapper = { className = "summary-icon-extra" } } }

[[root.children.node.children]]
mount = "content"
[root.children.node.children.node]
node_id = "accordion_summary_content"
kind = "native"
type = "Label"
control_id = "AccordionSummaryContent"
props = { text = "Expanded" }

[[root.children.node.children]]
mount = "expandIconWrapper"
[root.children.node.children.node]
node_id = "accordion_summary_icon"
kind = "native"
type = "Icon"
control_id = "AccordionSummaryIcon"
props = { icon = "expand_more" }

[[root.children]]
[root.children.node]
node_id = "accordion_actions"
kind = "native"
type = "AccordionActions"
control_id = "AccordionActionsRoot"
props = { disableSpacing = false }

[[root.children]]
[root.children.node]
node_id = "dialog_actions"
kind = "native"
type = "DialogActions"
control_id = "DialogActionsRoot"
props = { disableSpacing = false }

[[root.children]]
[root.children.node]
node_id = "dialog_content"
kind = "native"
type = "DialogContent"
control_id = "DialogContentRoot"
props = { dividers = true }

[[root.children]]
[root.children.node]
node_id = "mobile_stepper"
kind = "native"
type = "MobileStepper"
control_id = "MobileStepperRoot"
props = { activeStep = 1, steps = 3, position = "top", variant = "dots", slotProps = { dotActive = { className = "dot-active-extra" }, progress = { className = "progress-extra" } } }

[[root.children.node.children]]
mount = "dotActive"
[root.children.node.children.node]
node_id = "mobile_stepper_dot"
kind = "native"
type = "Label"
control_id = "MobileStepperDot"
props = { text = "Dot" }

[[root.children.node.children]]
mount = "progress"
[root.children.node.children.node]
node_id = "mobile_stepper_progress"
kind = "native"
type = "Progress"
control_id = "MobileStepperProgress"
props = { value = 0.5 }

[[root.children]]
[root.children.node]
node_id = "speed_dial_action"
kind = "native"
type = "SpeedDialAction"
control_id = "SpeedDialActionRoot"
props = { icon = "add", tooltipOpen = true, tooltipPlacement = "right", open = false, slotProps = { fab = { className = "fab-extra" }, staticTooltipLabel = { className = "tooltip-label-extra" } } }

[[root.children.node.children]]
mount = "fab"
[root.children.node.children.node]
node_id = "speed_dial_fab"
kind = "native"
type = "IconButton"
control_id = "SpeedDialFab"
props = { icon = "add" }

[[root.children.node.children]]
mount = "staticTooltipLabel"
[root.children.node.children.node]
node_id = "speed_dial_tooltip_label"
kind = "native"
type = "Label"
control_id = "SpeedDialTooltipLabel"
props = { text = "Create" }

[[root.children]]
[root.children.node]
node_id = "speed_dial_icon"
kind = "native"
type = "SpeedDialIcon"
control_id = "SpeedDialIconRoot"
props = { icon = "add", open = true, openIcon = "close" }

[[root.children.node.children]]
mount = "icon"
[root.children.node.children.node]
node_id = "speed_dial_icon_icon"
kind = "native"
type = "Icon"
control_id = "SpeedDialIconIcon"
props = { icon = "add" }

[[root.children.node.children]]
mount = "openIcon"
[root.children.node.children.node]
node_id = "speed_dial_open_icon"
kind = "native"
type = "Icon"
control_id = "SpeedDialOpenIcon"
props = { icon = "close" }

[[root.children]]
[root.children.node]
node_id = "swipeable_drawer"
kind = "native"
type = "SwipeableDrawer"
control_id = "SwipeableDrawerRoot"
props = { anchor = "right", variant = "temporary", open = true, slotProps = { paper = { className = "drawer-paper-extra" }, swipeArea = { className = "swipe-area-extra" } } }

[[root.children.node.children]]
mount = "paper"
[root.children.node.children.node]
node_id = "swipeable_drawer_paper"
kind = "native"
type = "Panel"
control_id = "SwipeableDrawerPaper"
props = { text = "Drawer" }

[[root.children.node.children]]
mount = "swipeArea"
[root.children.node.children.node]
node_id = "swipeable_drawer_area"
kind = "native"
type = "Panel"
control_id = "SwipeableDrawerSwipeArea"
props = { text = "Swipe" }

[[root.children]]
[root.children.node]
node_id = "tab_scroll_button"
kind = "native"
type = "TabScrollButton"
control_id = "TabScrollButtonRoot"
props = { direction = "right", orientation = "vertical", disabled = true }
"##;

#[test]
fn mui_surface_child_utility_classes_match_local_material_contracts() {
    let style = UiAssetLoader::load_toml_str(SURFACE_CHILD_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(SURFACE_CHILD_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_surface_child_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let summary = find_node(root, "AccordionSummaryRoot");
    assert_eq!(
        str_attr(summary, "surface_variant"),
        Some("accordion-summary-active")
    );
    assert_classes(
        summary,
        &[
            "MuiAccordionSummary-root",
            "MuiAccordionSummary-gutters",
            "MuiAccordionSummary-expanded",
            "MuiAccordionSummary-focusVisible",
            "Mui-expanded",
            "Mui-focusVisible",
            "summary-focus-extra",
        ],
    );
    assert_no_classes(
        summary,
        &[
            "MuiAccordionSummary-colorPrimary",
            "MuiAccordionSummary-sizeMedium",
        ],
    );

    let summary_content = find_node(root, "AccordionSummaryContent");
    assert_eq!(
        str_attr(summary_content, "text_tone"),
        Some("accordion-content")
    );
    assert_classes(
        summary_content,
        &[
            "MuiAccordionSummary-content",
            "Mui-expanded",
            "summary-content-extra",
        ],
    );

    let summary_icon = find_node(root, "AccordionSummaryIcon");
    assert_eq!(str_attr(summary_icon, "text_tone"), Some("accordion-icon"));
    assert_classes(
        summary_icon,
        &[
            "MuiAccordionSummary-expandIconWrapper",
            "Mui-expanded",
            "summary-icon-extra",
        ],
    );

    assert_eq!(
        str_attr(find_node(root, "AccordionActionsRoot"), "surface_variant"),
        Some("accordion-actions")
    );
    assert_eq!(
        str_attr(find_node(root, "DialogActionsRoot"), "surface_variant"),
        Some("dialog-actions")
    );
    assert_eq!(
        str_attr(find_node(root, "DialogContentRoot"), "surface_variant"),
        Some("dialog-content-dividers")
    );

    let stepper = find_node(root, "MobileStepperRoot");
    assert_eq!(
        str_attr(stepper, "surface_variant"),
        Some("mobile-stepper-top")
    );
    assert_classes(
        stepper,
        &["MuiMobileStepper-root", "MuiMobileStepper-positionTop"],
    );
    let dot = find_node(root, "MobileStepperDot");
    assert_eq!(str_attr(dot, "validation_level"), Some("active-dot"));
    assert_classes(
        dot,
        &[
            "MuiMobileStepper-dot",
            "MuiMobileStepper-dotActive",
            "dot-active-extra",
        ],
    );
    assert_eq!(
        str_attr(find_node(root, "MobileStepperProgress"), "surface_variant"),
        Some("mobile-progress")
    );

    let speed_action = find_node(root, "SpeedDialActionRoot");
    assert_eq!(
        str_attr(speed_action, "surface_variant"),
        Some("speed-action-tooltip")
    );
    assert_classes(
        speed_action,
        &[
            "MuiSpeedDialAction-staticTooltip",
            "MuiSpeedDialAction-staticTooltipClosed",
            "MuiSpeedDialAction-tooltipPlacementRight",
        ],
    );
    let fab = find_node(root, "SpeedDialFab");
    assert_eq!(str_attr(fab, "text_tone"), Some("fab-closed"));
    assert_classes(
        fab,
        &[
            "MuiSpeedDialAction-fab",
            "MuiSpeedDialAction-fabClosed",
            "fab-extra",
        ],
    );
    assert_eq!(
        str_attr(find_node(root, "SpeedDialTooltipLabel"), "text_tone"),
        Some("tooltip-label")
    );

    let icon = find_node(root, "SpeedDialIconIcon");
    assert_eq!(str_attr(icon, "text_tone"), Some("speed-icon-open"));
    assert_classes(
        icon,
        &[
            "MuiSpeedDialIcon-icon",
            "MuiSpeedDialIcon-iconOpen",
            "MuiSpeedDialIcon-iconWithOpenIconOpen",
        ],
    );
    let open_icon = find_node(root, "SpeedDialOpenIcon");
    assert_eq!(str_attr(open_icon, "text_tone"), Some("speed-open-icon"));
    assert_classes(
        open_icon,
        &["MuiSpeedDialIcon-openIcon", "MuiSpeedDialIcon-openIconOpen"],
    );

    let drawer = find_node(root, "SwipeableDrawerRoot");
    assert_eq!(
        str_attr(drawer, "surface_variant"),
        Some("swipeable-drawer-root")
    );
    assert_classes(
        drawer,
        &[
            "MuiSwipeableDrawer-root",
            "MuiDrawer-root",
            "MuiDrawer-anchorRight",
            "MuiDrawer-modal",
        ],
    );
    assert_eq!(
        str_attr(find_node(root, "SwipeableDrawerPaper"), "surface_variant"),
        Some("swipeable-drawer-paper")
    );
    assert_eq!(
        str_attr(
            find_node(root, "SwipeableDrawerSwipeArea"),
            "surface_variant"
        ),
        Some("swipe-area")
    );

    let tab_scroll = find_node(root, "TabScrollButtonRoot");
    assert_eq!(
        str_attr(tab_scroll, "validation_level"),
        Some("tab-scroll-disabled")
    );
    assert_classes(
        tab_scroll,
        &[
            "MuiTabScrollButton-root",
            "MuiTabScrollButton-vertical",
            "MuiTabScrollButton-disabled",
            "Mui-disabled",
        ],
    );
}

fn find_node<'a>(node: &'a UiTemplateNode, control_id: &str) -> &'a UiTemplateNode {
    if node.control_id.as_deref() == Some(control_id) {
        return node;
    }
    node.children
        .iter()
        .find_map(|child| find_node_opt(child, control_id))
        .unwrap_or_else(|| panic!("missing node `{control_id}`"))
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

fn assert_classes(node: &UiTemplateNode, expected: &[&str]) {
    for class_name in expected {
        assert!(
            node.classes.iter().any(|value| value == class_name),
            "missing {class_name} in {:?}",
            node.classes
        );
    }
}

fn assert_no_classes(node: &UiTemplateNode, unexpected: &[&str]) {
    for class_name in unexpected {
        assert!(
            !node.classes.iter().any(|value| value == class_name),
            "unexpected {class_name} in {:?}",
            node.classes
        );
    }
}
