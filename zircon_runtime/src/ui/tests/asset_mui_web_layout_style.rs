use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const LAYOUT_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_layout_style"
version = 1
display_name = "MUI Web Layout Style"

[[stylesheets]]
id = "mui_web_layout"

[[stylesheets.rules]]
selector = ".MuiContainer-maxWidthMd.MuiContainer-fixed.MuiContainer-disableGutters"
set = { self = { surface_variant = "container" } }

[[stylesheets.rules]]
selector = ".MuiGrid-container.MuiGrid-spacing-xs-2.MuiGrid-direction-xs-row-reverse.MuiGrid-wrap-xs-nowrap"
set = { self = { text_align = "center" } }

[[stylesheets.rules]]
selector = ".MuiGrid-container.MuiGrid-columns-md-16.MuiGrid-rowSpacing-sm-1.MuiGrid-columnSpacing-md-3.MuiGrid-direction-md-row.MuiGrid-wrap-xs-wrap-reverse"
set = { self = { surface_variant = "responsive-grid" } }

[[stylesheets.rules]]
selector = ".MuiGrid-grid-xs-6"
set = { self = { border_width = 6.0 } }

[[stylesheets.rules]]
selector = ".MuiGrid-grid-xs-12.MuiGrid-grid-md-6.MuiGrid-offset-md-2"
set = { self = { validation_level = "responsive-grid-item" } }

[[stylesheets.rules]]
selector = ".MuiStack-root.stack-extra"
set = { self = { surface_variant = "stack" } }

[[stylesheets.rules]]
selector = ".MuiStack-root.MuiStack-direction-xs-column.MuiStack-direction-md-row.MuiStack-spacing-xs-1.MuiStack-spacing-md-3.MuiStack-useFlexGap"
set = { self = { validation_level = "responsive-stack" } }

[[stylesheets.rules]]
selector = ".MuiUseMediaQuery-root.MuiUseMediaQuery-match.MuiUseMediaQuery-noSsr"
set = { self = { text_tone = "media-query-matched" } }

[[stylesheets.rules]]
selector = ".MuiMasonry-root"
set = { self = { overflow = "scroll" } }

[[stylesheets.rules]]
selector = ".MuiMasonry-root.MuiMasonry-columnsConfigured.MuiMasonry-spacingConfigured.MuiMasonry-sequential.MuiMasonry-ssrDefaults"
set = { self = { surface_variant = "masonry-configured" } }

[[stylesheets.rules]]
selector = ".MuiMasonry-root.responsive-masonry.MuiMasonry-columnsConfigured.MuiMasonry-spacingConfigured"
set = { self = { validation_level = "masonry-responsive" } }

[[stylesheets.rules]]
selector = ".MuiCollapse-horizontal.MuiCollapse-entered"
set = { self = { surface_variant = "collapse-entered" } }

[[stylesheets.rules]]
selector = ".MuiCollapse-wrapper.MuiCollapse-horizontal.collapse-wrapper-extra"
set = { self = { text_tone = "warning" } }

[[stylesheets.rules]]
selector = ".MuiCollapse-hidden"
set = { self = { validation_level = "hidden" } }

[[stylesheets.rules]]
selector = ".MuiClickAwayListener-root"
set = { self = { surface_variant = "click-away" } }

[[stylesheets.rules]]
selector = ".MuiPortal-root"
set = { self = { portal_layer = "layout-test" } }

[[stylesheets.rules]]
selector = ".MuiNoSsr-root"
set = { self = { text_tone = "muted" } }

[[stylesheets.rules]]
selector = ".MuiCssBaseline-root.baseline-extra"
set = { self = { color_scheme = "enabled" } }

[[stylesheets.rules]]
selector = ".MuiInitColorSchemeScript-root"
set = { self = { modeStorageKey = "test-mode" } }

[[stylesheets.rules]]
selector = ".MuiUseMediaQuery-root"
set = { self = { matches = true } }
"##;

const LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_layout_style_layout"
version = 1
display_name = "MUI Web Layout Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_layout_style.ui"]

[root]
node_id = "layout_root"
kind = "native"
type = "VerticalBox"
control_id = "LayoutRoot"

[[root.children]]
[root.children.node]
node_id = "container"
kind = "native"
type = "Container"
control_id = "ContainerRoot"
props = { maxWidth = "md", fixed = true, disableGutters = true, mui_color = "primary", mui_size = "medium" }

[[root.children]]
[root.children.node]
node_id = "grid_container"
kind = "native"
type = "Grid"
control_id = "GridContainer"
props = { container = true, spacing = "2", direction = "row-reverse", wrap = "nowrap" }

[[root.children]]
[root.children.node]
node_id = "grid_item"
kind = "native"
type = "Grid"
control_id = "GridItem"
props = { size = "6" }

[[root.children]]
[root.children.node]
node_id = "responsive_grid_container"
kind = "native"
type = "Grid"
control_id = "ResponsiveGridContainer"
props = { container = true, columns = { xs = 4, md = 16 }, rowSpacing = { xs = 0, sm = 1 }, columnSpacing = { xs = 1, md = 3 }, direction = { xs = "column", md = "row" }, wrap = "wrap-reverse" }

[[root.children]]
[root.children.node]
node_id = "responsive_grid_item"
kind = "native"
type = "Grid"
control_id = "ResponsiveGridItem"
props = { size = { xs = 12, md = 6 }, offset = { md = 2 } }

[[root.children]]
[root.children.node]
node_id = "stack"
kind = "native"
type = "Stack"
control_id = "StackRoot"
props = { className = "stack-extra", direction = "row", spacing = "2", useFlexGap = true }

[[root.children]]
[root.children.node]
node_id = "responsive_stack"
kind = "native"
type = "Stack"
control_id = "ResponsiveStack"
props = { direction = { xs = "column", md = "row" }, spacing = { xs = 1, md = 3 }, useFlexGap = true }

[[root.children]]
[root.children.node]
node_id = "masonry"
kind = "native"
type = "Masonry"
control_id = "MasonryRoot"
props = { columns = 4, spacing = "1", sequential = true, defaultColumns = 4, defaultHeight = "180", defaultSpacing = "1" }

[[root.children]]
[root.children.node]
node_id = "responsive_masonry"
kind = "native"
type = "Masonry"
control_id = "ResponsiveMasonryRoot"
props = { className = "responsive-masonry", columns = { xs = 1, md = 4 }, spacing = [1, 2] }

[[root.children]]
[root.children.node]
node_id = "collapse_entered"
kind = "native"
type = "Collapse"
control_id = "CollapseEntered"
props = { orientation = "horizontal", transition_status = "entered", in = true, slotProps = { wrapper = { className = "collapse-wrapper-extra" } } }

[[root.children.node.children]]
mount = "wrapper"
[root.children.node.children.node]
node_id = "collapse_wrapper"
kind = "native"
type = "Label"
control_id = "CollapseWrapper"
props = { text = "Wrapper" }

[[root.children]]
[root.children.node]
node_id = "collapse_hidden"
kind = "native"
type = "Collapse"
control_id = "CollapseHidden"
props = { transition_status = "exited", in = false, collapsedSize = "0px" }

[[root.children]]
[root.children.node]
node_id = "click_away"
kind = "native"
type = "ClickAwayListener"
control_id = "ClickAwayRoot"
props = { disableReactTree = true, mouseEvent = "onPointerDown", touchEvent = "false" }

[[root.children]]
[root.children.node]
node_id = "portal"
kind = "native"
type = "Portal"
control_id = "PortalRoot"
props = { disablePortal = true, container_id = "overlay-root" }

[[root.children]]
[root.children.node]
node_id = "no_ssr"
kind = "native"
type = "NoSsr"
control_id = "NoSsrRoot"
props = { defer = true }

[[root.children]]
[root.children.node]
node_id = "css_baseline"
kind = "native"
type = "CssBaseline"
control_id = "CssBaselineRoot"
props = { enableColorScheme = true, className = "baseline-extra" }

[[root.children]]
[root.children.node]
node_id = "init_script"
kind = "native"
type = "InitColorSchemeScript"
control_id = "InitScriptRoot"
props = { defaultMode = "dark", attribute = "data-mui-color-scheme" }

[[root.children]]
[root.children.node]
node_id = "media_query"
kind = "native"
type = "UseMediaQuery"
control_id = "MediaQueryRoot"
props = { query = "(min-width: 900px)", defaultMatches = false, matches = true, noSsr = true }
"##;

#[test]
fn mui_web_layout_utility_classes_match_local_material_contracts() {
    let style = UiAssetLoader::load_toml_str(LAYOUT_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_layout_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let container = find_node(root, "ContainerRoot");
    assert_eq!(str_attr(container, "surface_variant"), Some("container"));
    assert_classes(
        container,
        &[
            "MuiContainer-root",
            "MuiContainer-maxWidthMd",
            "MuiContainer-fixed",
            "MuiContainer-disableGutters",
        ],
    );
    assert_not_classes(
        container,
        &["MuiContainer-colorPrimary", "MuiContainer-sizeMedium"],
    );

    let grid_container = find_node(root, "GridContainer");
    assert_eq!(str_attr(grid_container, "text_align"), Some("center"));
    assert_classes(
        grid_container,
        &[
            "MuiGrid-root",
            "MuiGrid-container",
            "MuiGrid-spacing-xs-2",
            "MuiGrid-direction-xs-row-reverse",
            "MuiGrid-wrap-xs-nowrap",
        ],
    );

    let grid_item = find_node(root, "GridItem");
    assert_eq!(float_attr(grid_item, "border_width"), Some(6.0));
    assert_classes(grid_item, &["MuiGrid-root", "MuiGrid-grid-xs-6"]);

    let responsive_grid_container = find_node(root, "ResponsiveGridContainer");
    assert_eq!(
        str_attr(responsive_grid_container, "surface_variant"),
        Some("responsive-grid")
    );
    assert_classes(
        responsive_grid_container,
        &[
            "MuiGrid-root",
            "MuiGrid-container",
            "MuiGrid-columns-xs-4",
            "MuiGrid-columns-md-16",
            "MuiGrid-rowSpacing-sm-1",
            "MuiGrid-columnSpacing-xs-1",
            "MuiGrid-columnSpacing-md-3",
            "MuiGrid-direction-xs-column",
            "MuiGrid-direction-md-row",
            "MuiGrid-wrap-xs-wrap-reverse",
        ],
    );
    assert_not_classes(responsive_grid_container, &["MuiGrid-rowSpacing-xs-0"]);

    let responsive_grid_item = find_node(root, "ResponsiveGridItem");
    assert_eq!(
        str_attr(responsive_grid_item, "validation_level"),
        Some("responsive-grid-item")
    );
    assert_classes(
        responsive_grid_item,
        &[
            "MuiGrid-root",
            "MuiGrid-grid-xs-12",
            "MuiGrid-grid-md-6",
            "MuiGrid-offset-md-2",
        ],
    );

    let stack = find_node(root, "StackRoot");
    assert_eq!(str_attr(stack, "surface_variant"), Some("stack"));
    assert_classes(
        stack,
        &[
            "MuiStack-root",
            "stack-extra",
            "MuiStack-direction-xs-row",
            "MuiStack-spacing-xs-2",
            "MuiStack-useFlexGap",
        ],
    );

    let responsive_stack = find_node(root, "ResponsiveStack");
    assert_eq!(
        str_attr(responsive_stack, "validation_level"),
        Some("responsive-stack")
    );
    assert_classes(
        responsive_stack,
        &[
            "MuiStack-root",
            "MuiStack-direction-xs-column",
            "MuiStack-direction-md-row",
            "MuiStack-spacing-xs-1",
            "MuiStack-spacing-md-3",
            "MuiStack-useFlexGap",
        ],
    );

    let masonry = find_node(root, "MasonryRoot");
    assert_eq!(str_attr(masonry, "overflow"), Some("scroll"));
    assert_eq!(
        str_attr(masonry, "surface_variant"),
        Some("masonry-configured")
    );
    assert_classes(
        masonry,
        &[
            "MuiMasonry-root",
            "MuiMasonry-columnsConfigured",
            "MuiMasonry-spacingConfigured",
            "MuiMasonry-sequential",
            "MuiMasonry-ssrDefaults",
        ],
    );

    let responsive_masonry = find_node(root, "ResponsiveMasonryRoot");
    assert_eq!(
        str_attr(responsive_masonry, "validation_level"),
        Some("masonry-responsive")
    );
    assert_classes(
        responsive_masonry,
        &[
            "MuiMasonry-root",
            "responsive-masonry",
            "MuiMasonry-columnsConfigured",
            "MuiMasonry-spacingConfigured",
        ],
    );
    assert_not_classes(
        responsive_masonry,
        &["MuiMasonry-sequential", "MuiMasonry-ssrDefaults"],
    );

    let collapse_entered = find_node(root, "CollapseEntered");
    assert_eq!(
        str_attr(collapse_entered, "surface_variant"),
        Some("collapse-entered")
    );
    assert_classes(
        collapse_entered,
        &[
            "MuiCollapse-root",
            "MuiCollapse-horizontal",
            "MuiCollapse-entered",
        ],
    );
    let collapse_wrapper = find_node(root, "CollapseWrapper");
    assert_eq!(str_attr(collapse_wrapper, "text_tone"), Some("warning"));
    assert_classes(
        collapse_wrapper,
        &[
            "MuiCollapse-wrapper",
            "MuiCollapse-horizontal",
            "collapse-wrapper-extra",
        ],
    );

    let collapse_hidden = find_node(root, "CollapseHidden");
    assert_eq!(
        str_attr(collapse_hidden, "validation_level"),
        Some("hidden")
    );
    assert_classes(
        collapse_hidden,
        &[
            "MuiCollapse-root",
            "MuiCollapse-vertical",
            "MuiCollapse-hidden",
        ],
    );

    assert_eq!(
        str_attr(find_node(root, "ClickAwayRoot"), "surface_variant"),
        Some("click-away")
    );
    assert_eq!(
        str_attr(find_node(root, "PortalRoot"), "portal_layer"),
        Some("layout-test")
    );
    assert_eq!(
        str_attr(find_node(root, "NoSsrRoot"), "text_tone"),
        Some("muted")
    );
    assert_eq!(
        str_attr(find_node(root, "CssBaselineRoot"), "color_scheme"),
        Some("enabled")
    );
    assert_eq!(
        str_attr(find_node(root, "InitScriptRoot"), "modeStorageKey"),
        Some("test-mode")
    );
    assert_eq!(
        bool_attr(find_node(root, "MediaQueryRoot"), "matches"),
        Some(true)
    );
    assert_eq!(
        str_attr(find_node(root, "MediaQueryRoot"), "text_tone"),
        Some("media-query-matched")
    );
    assert_classes(
        find_node(root, "MediaQueryRoot"),
        &[
            "MuiUseMediaQuery-root",
            "MuiUseMediaQuery-match",
            "MuiUseMediaQuery-noSsr",
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

fn bool_attr(node: &UiTemplateNode, name: &str) -> Option<bool> {
    node.attributes.get(name).and_then(Value::as_bool)
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

fn assert_not_classes(node: &UiTemplateNode, unexpected: &[&str]) {
    for class_name in unexpected {
        assert!(
            !node.classes.iter().any(|value| value == class_name),
            "unexpected {class_name} in {:?}",
            node.classes
        );
    }
}
