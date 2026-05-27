use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const BADGE_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_badge_style"
version = 1
display_name = "MUI Web Badge Style"

[[stylesheets]]
id = "mui_web_badge"

[[stylesheets.rules]]
selector = ".MuiBadge-badge.MuiBadge-standard.MuiBadge-invisible.MuiBadge-anchorOriginTopRight.MuiBadge-anchorOriginTopRightRectangular.MuiBadge-overlapRectangular"
set = { self = { validation_level = "zero-hidden" } }

[[stylesheets.rules]]
selector = ".MuiBadge-badge.MuiBadge-standard.MuiBadge-anchorOriginTopRightRectangular.MuiBadge-overlapRectangular.zero-visible"
set = { self = { validation_level = "zero-visible" } }

[[stylesheets.rules]]
selector = ".MuiBadge-badge.MuiBadge-standard.MuiBadge-anchorOriginTopRightRectangular.MuiBadge-overlapRectangular.string-zero-visible"
set = { self = { validation_level = "string-zero-visible" } }
"##;

const BADGE_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_badge_style_layout"
version = 1
display_name = "MUI Web Badge Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_badge_style.ui"]

[root]
node_id = "badge_style_root"
kind = "native"
type = "VerticalBox"
control_id = "BadgeStyleRoot"

[[root.children]]
[root.children.node]
node_id = "hidden_zero_badge"
kind = "native"
type = "Badge"
control_id = "HiddenZeroBadge"
props = { badgeContent = 0, showZero = false, variant = "standard", overlap = "rectangular", anchorOrigin = { vertical = "top", horizontal = "right" } }

[[root.children.node.children]]
mount = "badge"
[root.children.node.children.node]
node_id = "hidden_zero_badge_slot"
kind = "native"
type = "Label"
control_id = "HiddenZeroBadgeSlot"
props = { text = "0" }

[[root.children]]
[root.children.node]
node_id = "visible_zero_badge"
kind = "native"
type = "Badge"
control_id = "VisibleZeroBadge"
props = { badgeContent = 0, showZero = true, variant = "standard", overlap = "rectangular", anchorOrigin = { vertical = "top", horizontal = "right" }, slotProps = { badge = { className = "zero-visible" } } }

[[root.children.node.children]]
mount = "badge"
[root.children.node.children.node]
node_id = "visible_zero_badge_slot"
kind = "native"
type = "Label"
control_id = "VisibleZeroBadgeSlot"
props = { text = "0" }

[[root.children]]
[root.children.node]
node_id = "missing_content_badge"
kind = "native"
type = "Badge"
control_id = "MissingContentBadge"
props = { variant = "standard", overlap = "rectangular", anchorOrigin = { vertical = "top", horizontal = "right" } }

[[root.children.node.children]]
mount = "badge"
[root.children.node.children.node]
node_id = "missing_content_badge_slot"
kind = "native"
type = "Label"
control_id = "MissingContentBadgeSlot"
props = { text = "" }

[[root.children]]
[root.children.node]
node_id = "string_zero_badge"
kind = "native"
type = "Badge"
control_id = "StringZeroBadge"
props = { badgeContent = "0", variant = "standard", overlap = "rectangular", anchorOrigin = { vertical = "top", horizontal = "right" }, slotProps = { badge = { className = "string-zero-visible" } } }

[[root.children.node.children]]
mount = "badge"
[root.children.node.children.node]
node_id = "string_zero_badge_slot"
kind = "native"
type = "Label"
control_id = "StringZeroBadgeSlot"
props = { text = "0" }
"##;

#[test]
fn mui_badge_zero_content_slot_invisibility_matches_local_material_contract() {
    let style = UiAssetLoader::load_toml_str(BADGE_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(BADGE_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_badge_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let hidden_slot = find_node(root, "HiddenZeroBadgeSlot");
    let visible_slot = find_node(root, "VisibleZeroBadgeSlot");
    let missing_content_slot = find_node(root, "MissingContentBadgeSlot");
    let string_zero_slot = find_node(root, "StringZeroBadgeSlot");

    assert_eq!(
        str_attr(hidden_slot, "validation_level"),
        Some("zero-hidden")
    );
    assert_classes(
        hidden_slot,
        &[
            "MuiBadge-badge",
            "MuiBadge-standard",
            "MuiBadge-invisible",
            "MuiBadge-anchorOriginTopRight",
            "MuiBadge-anchorOriginTopRightRectangular",
            "MuiBadge-overlapRectangular",
        ],
    );
    assert!(
        str_attr(hidden_slot, "component_variant").is_some_and(|value| {
            value.split_whitespace().any(|part| part == "muiBadgeSlot")
                && value.split_whitespace().any(|part| part == "invisible")
        }),
        "zero-count hidden Badge slot should carry retained invisible metadata"
    );

    assert_eq!(
        str_attr(visible_slot, "validation_level"),
        Some("zero-visible")
    );
    assert_classes(
        visible_slot,
        &[
            "MuiBadge-badge",
            "MuiBadge-standard",
            "MuiBadge-anchorOriginTopRight",
            "MuiBadge-anchorOriginTopRightRectangular",
            "MuiBadge-overlapRectangular",
            "zero-visible",
        ],
    );
    assert_no_classes(visible_slot, &["MuiBadge-invisible"]);

    assert_eq!(
        str_attr(missing_content_slot, "validation_level"),
        Some("zero-hidden")
    );
    assert_classes(
        missing_content_slot,
        &[
            "MuiBadge-badge",
            "MuiBadge-standard",
            "MuiBadge-invisible",
            "MuiBadge-anchorOriginTopRight",
            "MuiBadge-anchorOriginTopRightRectangular",
            "MuiBadge-overlapRectangular",
        ],
    );

    assert_eq!(
        str_attr(string_zero_slot, "validation_level"),
        Some("string-zero-visible")
    );
    assert_classes(
        string_zero_slot,
        &[
            "MuiBadge-badge",
            "MuiBadge-standard",
            "MuiBadge-anchorOriginTopRight",
            "MuiBadge-anchorOriginTopRightRectangular",
            "MuiBadge-overlapRectangular",
            "string-zero-visible",
        ],
    );
    assert_no_classes(string_zero_slot, &["MuiBadge-invisible"]);
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
