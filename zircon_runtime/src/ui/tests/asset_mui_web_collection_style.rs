use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const COLLECTION_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_collection_style"
version = 1
display_name = "MUI Web Collection Style"

[[stylesheets]]
id = "mui_web_collection"

[[stylesheets.rules]]
selector = ".MuiListItem-dense.MuiListItem-gutters.MuiListItem-padding.MuiListItem-divider.MuiListItem-alignItemsFlexStart"
set = { self = { surface_variant = "list-row" } }

[[stylesheets.rules]]
selector = ".MuiListItem-secondaryAction.list-secondary-extra"
set = { self = { text_tone = "warning" } }

[[stylesheets.rules]]
selector = ".MuiListItemButton-dense.MuiListItemButton-gutters.MuiListItemButton-divider.MuiListItemButton-alignItemsFlexStart.MuiListItemButton-selected.Mui-selected.MuiListItemButton-focusVisible.Mui-focusVisible"
set = { self = { validation_level = "info" } }

[[stylesheets.rules]]
selector = ".MuiListItemAvatar-alignItemsFlexStart"
set = { self = { surface_variant = "avatar-slot" } }

[[stylesheets.rules]]
selector = ".MuiListItemIcon-alignItemsFlexStart"
set = { self = { text_tone = "icon" } }

[[stylesheets.rules]]
selector = ".MuiListItemSecondaryAction-disableGutters"
set = { self = { text_align = "right" } }

[[stylesheets.rules]]
selector = ".MuiListItemText-inset.MuiListItemText-dense.MuiListItemText-multiline"
set = { self = { surface_variant = "text-stack" } }

[[stylesheets.rules]]
selector = ".MuiListItemText-primary.primary-extra"
set = { self = { text_tone = "primary-row" } }

[[stylesheets.rules]]
selector = ".MuiListSubheader-colorPrimary.MuiListSubheader-gutters.MuiListSubheader-inset.MuiListSubheader-sticky"
set = { self = { text_tone = "subheader" } }

[[stylesheets.rules]]
selector = ".MuiImageListItem-masonry"
set = { self = { overflow = "masonry" } }

[[stylesheets.rules]]
selector = ".MuiImageListItem-img.img-extra"
set = { self = { surface_variant = "image" } }

[[stylesheets.rules]]
selector = ".MuiImageListItemBar-positionTop.MuiImageListItemBar-actionPositionLeft"
set = { self = { surface_variant = "image-bar" } }

[[stylesheets.rules]]
selector = ".MuiImageListItemBar-title.bar-title-extra"
set = { self = { text_tone = "bar-title" } }

[[stylesheets.rules]]
selector = ".MuiTableRow-selected.MuiTableRow-hover.MuiTableRow-head.Mui-selected"
set = { self = { validation_level = "selected-row" } }

[[stylesheets.rules]]
selector = ".MuiTableCell-head.MuiTableCell-stickyHeader.MuiTableCell-alignRight.MuiTableCell-paddingCheckbox.MuiTableCell-sizeSmall"
set = { self = { text_align = "right" } }

[[stylesheets.rules]]
selector = ".MuiTableSortLabel-active.MuiTableSortLabel-directionDesc.Mui-active"
set = { self = { validation_level = "sort-active" } }

[[stylesheets.rules]]
selector = ".MuiTableSortLabel-icon.sort-icon-extra"
set = { self = { text_tone = "sort-icon" } }

[[stylesheets.rules]]
selector = ".MuiTablePagination-toolbar.pagination-toolbar-extra"
set = { self = { surface_variant = "pagination-toolbar" } }

[[stylesheets.rules]]
selector = ".MuiTablePaginationActions-root"
set = { self = { surface_variant = "pagination-actions" } }
"##;

const COLLECTION_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_collection_style_layout"
version = 1
display_name = "MUI Web Collection Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_collection_style.ui"]

[root]
node_id = "collection_root"
kind = "native"
type = "VerticalBox"
control_id = "CollectionRoot"

[[root.children]]
[root.children.node]
node_id = "list_item"
kind = "native"
type = "ListItem"
control_id = "ListItemRoot"
props = { alignItems = "flex-start", dense = true, divider = true, slotProps = { secondaryAction = { className = "list-secondary-extra" } } }

[[root.children.node.children]]
mount = "secondaryAction"
[root.children.node.children.node]
node_id = "list_item_secondary_slot"
kind = "native"
type = "Label"
control_id = "ListItemSecondarySlot"
props = { text = "More" }

[[root.children]]
[root.children.node]
node_id = "list_button"
kind = "native"
type = "ListItemButton"
control_id = "ListItemButtonRoot"
props = { alignItems = "flex-start", dense = true, divider = true, selected = true, focusVisible = true }

[[root.children]]
[root.children.node]
node_id = "list_avatar"
kind = "native"
type = "ListItemAvatar"
control_id = "ListItemAvatarRoot"
props = { alignItems = "flex-start" }

[[root.children]]
[root.children.node]
node_id = "list_icon"
kind = "native"
type = "ListItemIcon"
control_id = "ListItemIconRoot"
props = { icon = "folder", alignItems = "flex-start" }

[[root.children]]
[root.children.node]
node_id = "list_secondary_action"
kind = "native"
type = "ListItemSecondaryAction"
control_id = "ListSecondaryActionRoot"
props = { disableGutters = true }

[[root.children]]
[root.children.node]
node_id = "list_text"
kind = "native"
type = "ListItemText"
control_id = "ListItemTextRoot"
props = { primary = "Scene", secondary = "Ready", dense = true, inset = true, slotProps = { primary = { className = "primary-extra" } } }

[[root.children.node.children]]
mount = "primary"
[root.children.node.children.node]
node_id = "list_text_primary"
kind = "native"
type = "Label"
control_id = "ListItemTextPrimary"
props = { text = "Scene" }

[[root.children]]
[root.children.node]
node_id = "list_subheader"
kind = "native"
type = "ListSubheader"
control_id = "ListSubheaderRoot"
props = { text = "Assets", color = "primary", inset = true }

[[root.children]]
[root.children.node]
node_id = "image_list_item"
kind = "native"
type = "ImageListItem"
control_id = "ImageListItemRoot"
props = { src = "res://preview.png", variant = "masonry", slotProps = { img = { className = "img-extra" } } }

[[root.children.node.children]]
mount = "img"
[root.children.node.children.node]
node_id = "image_list_img"
kind = "native"
type = "Label"
control_id = "ImageListItemImg"
props = { text = "Img" }

[[root.children]]
[root.children.node]
node_id = "image_list_bar"
kind = "native"
type = "ImageListItemBar"
control_id = "ImageListItemBarRoot"
props = { title = "Preview", subtitle = "PNG", position = "top", actionPosition = "left", slotProps = { title = { className = "bar-title-extra" } } }

[[root.children.node.children]]
mount = "title"
[root.children.node.children.node]
node_id = "image_list_bar_title"
kind = "native"
type = "Label"
control_id = "ImageListItemBarTitle"
props = { text = "Preview" }

[[root.children]]
[root.children.node]
node_id = "table_row"
kind = "native"
type = "TableRow"
control_id = "TableRowRoot"
props = { variant = "head", hover = true, selected = true }

[[root.children]]
[root.children.node]
node_id = "table_cell"
kind = "native"
type = "TableCell"
control_id = "TableCellRoot"
props = { text = "Name", variant = "head", stickyHeader = true, align = "right", padding = "checkbox", size = "small" }

[[root.children]]
[root.children.node]
node_id = "table_sort_label"
kind = "native"
type = "TableSortLabel"
control_id = "TableSortLabelRoot"
props = { text = "Name", active = true, direction = "desc", slotProps = { icon = { className = "sort-icon-extra" } } }

[[root.children.node.children]]
mount = "icon"
[root.children.node.children.node]
node_id = "table_sort_icon"
kind = "native"
type = "Label"
control_id = "TableSortIcon"
props = { text = "Arrow" }

[[root.children]]
[root.children.node]
node_id = "table_pagination"
kind = "native"
type = "TablePagination"
control_id = "TablePaginationRoot"
props = { count = 120, page = 1, rowsPerPage = 25, slotProps = { toolbar = { className = "pagination-toolbar-extra" } } }

[[root.children.node.children]]
mount = "toolbar"
[root.children.node.children.node]
node_id = "table_pagination_toolbar"
kind = "native"
type = "Label"
control_id = "TablePaginationToolbar"
props = { text = "Toolbar" }

[[root.children]]
[root.children.node]
node_id = "table_pagination_actions"
kind = "native"
type = "TablePaginationActions"
control_id = "TablePaginationActionsRoot"
props = { count = 120, page = 1, rowsPerPage = 25, showFirstButton = true, showLastButton = true }
"##;

#[test]
fn mui_collection_utility_classes_match_local_material_contracts() {
    let style = UiAssetLoader::load_toml_str(COLLECTION_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(COLLECTION_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_collection_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let list_item = find_node(root, "ListItemRoot");
    assert_eq!(str_attr(list_item, "surface_variant"), Some("list-row"));
    assert_classes(
        list_item,
        &[
            "MuiListItem-root",
            "MuiListItem-dense",
            "MuiListItem-gutters",
            "MuiListItem-padding",
            "MuiListItem-divider",
            "MuiListItem-alignItemsFlexStart",
        ],
    );
    assert_no_classes(
        list_item,
        &["MuiListItem-colorPrimary", "MuiListItem-sizeMedium"],
    );
    let secondary_slot = find_node(root, "ListItemSecondarySlot");
    assert_eq!(str_attr(secondary_slot, "text_tone"), Some("warning"));
    assert_classes(
        secondary_slot,
        &["MuiListItem-secondaryAction", "list-secondary-extra"],
    );

    let list_button = find_node(root, "ListItemButtonRoot");
    assert_eq!(str_attr(list_button, "validation_level"), Some("info"));
    assert_classes(
        list_button,
        &[
            "MuiListItemButton-root",
            "MuiListItemButton-dense",
            "MuiListItemButton-gutters",
            "MuiListItemButton-divider",
            "MuiListItemButton-alignItemsFlexStart",
            "MuiListItemButton-selected",
            "MuiListItemButton-focusVisible",
            "Mui-selected",
            "Mui-focusVisible",
        ],
    );
    assert_no_classes(
        list_button,
        &[
            "MuiListItemButton-colorPrimary",
            "MuiListItemButton-sizeMedium",
        ],
    );

    assert_eq!(
        str_attr(find_node(root, "ListItemAvatarRoot"), "surface_variant"),
        Some("avatar-slot")
    );
    assert_eq!(
        str_attr(find_node(root, "ListItemIconRoot"), "text_tone"),
        Some("icon")
    );
    assert_eq!(
        str_attr(find_node(root, "ListSecondaryActionRoot"), "text_align"),
        Some("right")
    );

    let list_text = find_node(root, "ListItemTextRoot");
    assert_eq!(str_attr(list_text, "surface_variant"), Some("text-stack"));
    assert_classes(
        list_text,
        &[
            "MuiListItemText-root",
            "MuiListItemText-inset",
            "MuiListItemText-dense",
            "MuiListItemText-multiline",
        ],
    );
    let text_primary = find_node(root, "ListItemTextPrimary");
    assert_eq!(str_attr(text_primary, "text_tone"), Some("primary-row"));
    assert_classes(text_primary, &["MuiListItemText-primary", "primary-extra"]);

    let subheader = find_node(root, "ListSubheaderRoot");
    assert_eq!(str_attr(subheader, "text_tone"), Some("subheader"));
    assert_classes(
        subheader,
        &[
            "MuiListSubheader-root",
            "MuiListSubheader-colorPrimary",
            "MuiListSubheader-gutters",
            "MuiListSubheader-inset",
            "MuiListSubheader-sticky",
        ],
    );
    assert_no_classes(subheader, &["MuiListSubheader-sizeMedium"]);

    let image_item = find_node(root, "ImageListItemRoot");
    assert_eq!(str_attr(image_item, "overflow"), Some("masonry"));
    assert_classes(
        image_item,
        &["MuiImageListItem-root", "MuiImageListItem-masonry"],
    );
    let image = find_node(root, "ImageListItemImg");
    assert_eq!(str_attr(image, "surface_variant"), Some("image"));
    assert_classes(image, &["MuiImageListItem-img", "img-extra"]);

    let image_bar = find_node(root, "ImageListItemBarRoot");
    assert_eq!(str_attr(image_bar, "surface_variant"), Some("image-bar"));
    assert_classes(
        image_bar,
        &[
            "MuiImageListItemBar-root",
            "MuiImageListItemBar-positionTop",
            "MuiImageListItemBar-actionPositionLeft",
        ],
    );
    let bar_title = find_node(root, "ImageListItemBarTitle");
    assert_eq!(str_attr(bar_title, "text_tone"), Some("bar-title"));
    assert_classes(bar_title, &["MuiImageListItemBar-title", "bar-title-extra"]);

    let table_row = find_node(root, "TableRowRoot");
    assert_eq!(
        str_attr(table_row, "validation_level"),
        Some("selected-row")
    );
    assert_classes(
        table_row,
        &[
            "MuiTableRow-root",
            "MuiTableRow-selected",
            "MuiTableRow-hover",
            "MuiTableRow-head",
            "Mui-selected",
        ],
    );

    let table_cell = find_node(root, "TableCellRoot");
    assert_eq!(str_attr(table_cell, "text_align"), Some("right"));
    assert_classes(
        table_cell,
        &[
            "MuiTableCell-root",
            "MuiTableCell-head",
            "MuiTableCell-stickyHeader",
            "MuiTableCell-alignRight",
            "MuiTableCell-paddingCheckbox",
            "MuiTableCell-sizeSmall",
        ],
    );

    let sort_label = find_node(root, "TableSortLabelRoot");
    assert_eq!(
        str_attr(sort_label, "validation_level"),
        Some("sort-active")
    );
    assert_classes(
        sort_label,
        &[
            "MuiTableSortLabel-root",
            "MuiTableSortLabel-active",
            "MuiTableSortLabel-directionDesc",
            "Mui-active",
        ],
    );
    let sort_icon = find_node(root, "TableSortIcon");
    assert_eq!(str_attr(sort_icon, "text_tone"), Some("sort-icon"));
    assert_classes(sort_icon, &["MuiTableSortLabel-icon", "sort-icon-extra"]);

    let pagination_toolbar = find_node(root, "TablePaginationToolbar");
    assert_eq!(
        str_attr(pagination_toolbar, "surface_variant"),
        Some("pagination-toolbar")
    );
    assert_classes(
        pagination_toolbar,
        &["MuiTablePagination-toolbar", "pagination-toolbar-extra"],
    );
    assert_eq!(
        str_attr(
            find_node(root, "TablePaginationActionsRoot"),
            "surface_variant"
        ),
        Some("pagination-actions")
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
