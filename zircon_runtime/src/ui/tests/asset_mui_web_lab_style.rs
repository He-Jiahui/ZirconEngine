use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const LAB_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_lab_style"
version = 1
display_name = "MUI Web Lab Style"

[[stylesheets]]
id = "mui_web_lab"

[[stylesheets.rules]]
selector = ".MuiTabPanel-root.MuiTabPanel-hidden"
set = { self = { surface_variant = "tab-panel-hidden" } }

[[stylesheets.rules]]
selector = ".MuiTabList-root.tab-list-extra"
set = { self = { surface_variant = "tab-list" } }

[[stylesheets.rules]]
selector = ".MuiTimeline-root.MuiTimeline-positionAlternateReverse"
set = { self = { surface_variant = "timeline-alternate-reverse" } }

[[stylesheets.rules]]
selector = ".MuiTimelineItem-root.MuiTimelineItem-positionAlternateReverse.MuiTimelineItem-missingOppositeContent"
set = { self = { surface_variant = "timeline-item-missing-opposite" } }

[[stylesheets.rules]]
selector = ".MuiTimelineContent-root.MuiTimelineContent-positionRight.timeline-content-extra"
set = { self = { text_align = "left" } }

[[stylesheets.rules]]
selector = ".MuiTimelineOppositeContent-root.MuiTimelineOppositeContent-positionLeft"
set = { self = { text_align = "right" } }

[[stylesheets.rules]]
selector = ".MuiTimelineDot-root.MuiTimelineDot-outlined.MuiTimelineDot-outlinedSecondary.dot-extra"
set = { self = { validation_level = "timeline-dot" } }

[[stylesheets.rules]]
selector = ".MuiTimelineSeparator-root"
set = { self = { surface_variant = "timeline-separator" } }

[[stylesheets.rules]]
selector = ".MuiTimelineConnector-root"
set = { self = { surface_variant = "timeline-connector" } }

[[stylesheets.rules]]
selector = ".MuiTreeItem-root.Mui-selected.Mui-expanded"
set = { self = { validation_level = "tree-selected-expanded" } }
"##;

const LAB_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_lab_style_layout"
version = 1
display_name = "MUI Web Lab Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_lab_style.ui"]

[root]
node_id = "lab_root"
kind = "native"
type = "VerticalBox"
control_id = "LabRoot"

[[root.children]]
[root.children.node]
node_id = "tab_list"
kind = "native"
type = "TabList"
control_id = "TabListRoot"
props = { value = "main", orientation = "vertical", className = "tab-list-extra" }

[[root.children]]
[root.children.node]
node_id = "tab_panel"
kind = "native"
type = "TabPanel"
control_id = "TabPanelRoot"
props = { value = "settings", context_value = "main", keepMounted = true }

[[root.children]]
[root.children.node]
node_id = "timeline"
kind = "native"
type = "Timeline"
control_id = "TimelineRoot"
props = { position = "alternate-reverse" }

[[root.children]]
[root.children.node]
node_id = "timeline_item"
kind = "native"
type = "TimelineItem"
control_id = "TimelineItemRoot"
props = { position = "alternate-reverse", hasOppositeContent = false }

[[root.children]]
[root.children.node]
node_id = "timeline_content"
kind = "native"
type = "TimelineContent"
control_id = "TimelineContentRoot"
props = { position = "right", className = "timeline-content-extra" }

[[root.children]]
[root.children.node]
node_id = "timeline_opposite_content"
kind = "native"
type = "TimelineOppositeContent"
control_id = "TimelineOppositeContentRoot"
props = { position = "left" }

[[root.children]]
[root.children.node]
node_id = "timeline_dot"
kind = "native"
type = "TimelineDot"
control_id = "TimelineDotRoot"
props = { variant = "outlined", color = "secondary", className = "dot-extra" }

[[root.children]]
[root.children.node]
node_id = "timeline_separator"
kind = "native"
type = "TimelineSeparator"
control_id = "TimelineSeparatorRoot"
props = { }

[[root.children]]
[root.children.node]
node_id = "timeline_connector"
kind = "native"
type = "TimelineConnector"
control_id = "TimelineConnectorRoot"
props = { }

[[root.children]]
[root.children.node]
node_id = "tree_item"
kind = "native"
type = "TreeItem"
control_id = "TreeItemRoot"
props = { selected = true, expanded = true, label = "Root" }
"##;

#[test]
fn mui_lab_utility_classes_match_local_lab_contracts() {
    let style = UiAssetLoader::load_toml_str(LAB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(LAB_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_lab_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let tab_list = find_node(root, "TabListRoot");
    assert_eq!(str_attr(tab_list, "surface_variant"), Some("tab-list"));
    assert_classes(tab_list, &["MuiTabList-root", "tab-list-extra"]);

    let tab_panel = find_node(root, "TabPanelRoot");
    assert_eq!(
        str_attr(tab_panel, "surface_variant"),
        Some("tab-panel-hidden")
    );
    assert_classes(tab_panel, &["MuiTabPanel-root", "MuiTabPanel-hidden"]);

    let timeline = find_node(root, "TimelineRoot");
    assert_eq!(
        str_attr(timeline, "surface_variant"),
        Some("timeline-alternate-reverse")
    );
    assert_classes(
        timeline,
        &["MuiTimeline-root", "MuiTimeline-positionAlternateReverse"],
    );

    let item = find_node(root, "TimelineItemRoot");
    assert_eq!(
        str_attr(item, "surface_variant"),
        Some("timeline-item-missing-opposite")
    );
    assert_classes(
        item,
        &[
            "MuiTimelineItem-root",
            "MuiTimelineItem-positionAlternateReverse",
            "MuiTimelineItem-missingOppositeContent",
        ],
    );

    let content = find_node(root, "TimelineContentRoot");
    assert_eq!(str_attr(content, "text_align"), Some("left"));
    assert_classes(
        content,
        &[
            "MuiTimelineContent-root",
            "MuiTimelineContent-positionRight",
            "timeline-content-extra",
        ],
    );

    let opposite = find_node(root, "TimelineOppositeContentRoot");
    assert_eq!(str_attr(opposite, "text_align"), Some("right"));
    assert_classes(
        opposite,
        &[
            "MuiTimelineOppositeContent-root",
            "MuiTimelineOppositeContent-positionLeft",
        ],
    );

    let dot = find_node(root, "TimelineDotRoot");
    assert_eq!(str_attr(dot, "validation_level"), Some("timeline-dot"));
    assert_classes(
        dot,
        &[
            "MuiTimelineDot-root",
            "MuiTimelineDot-outlined",
            "MuiTimelineDot-outlinedSecondary",
            "dot-extra",
        ],
    );
    assert_no_classes(
        dot,
        &["MuiTimelineDot-colorSecondary", "MuiTimelineDot-sizeMedium"],
    );

    assert_eq!(
        str_attr(find_node(root, "TimelineSeparatorRoot"), "surface_variant"),
        Some("timeline-separator")
    );
    assert_eq!(
        str_attr(find_node(root, "TimelineConnectorRoot"), "surface_variant"),
        Some("timeline-connector")
    );

    let tree_item = find_node(root, "TreeItemRoot");
    assert_eq!(
        str_attr(tree_item, "validation_level"),
        Some("tree-selected-expanded")
    );
    assert_classes(
        tree_item,
        &["MuiTreeItem-root", "Mui-selected", "Mui-expanded"],
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
