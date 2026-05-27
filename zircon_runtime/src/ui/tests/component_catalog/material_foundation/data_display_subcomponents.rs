use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiRenderCapability, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_list_subcomponents(registry);
    assert_image_list_subcomponents(registry);
    assert_table_subcomponents(registry);
}

fn assert_list_subcomponents(registry: &UiComponentDescriptorRegistry) {
    let list_item = descriptor(registry, "ListItem");
    assert_enum_options(list_item, "alignItems", &["center", "flex-start"]);
    for prop in [
        "component",
        "dense",
        "disableGutters",
        "disablePadding",
        "divider",
        "secondaryAction",
    ] {
        assert_has_prop(list_item, prop);
    }
    for slot in ["content", "secondaryAction"] {
        assert_has_slot(list_item, slot);
    }

    let button = descriptor(registry, "ListItemButton");
    assert_enum_options(button, "alignItems", &["center", "flex-start"]);
    for prop in [
        "autoFocus",
        "component",
        "dense",
        "disableGutters",
        "divider",
        "focusVisibleClassName",
        "href",
        "selected",
        "to",
    ] {
        assert_has_prop(button, prop);
    }
    assert_has_event(button, UiComponentEventKind::SelectOption);
    assert_has_event(button, UiComponentEventKind::Commit);

    for component_id in ["ListItemAvatar", "ListItemIcon"] {
        let descriptor = descriptor(registry, component_id);
        assert_enum_options(descriptor, "alignItems", &["center", "flex-start"]);
        assert_has_slot(descriptor, "content");
    }
    assert!(descriptor(registry, "ListItemIcon")
        .required_render_capabilities
        .contains(&UiRenderCapability::Vector));

    let secondary_action = descriptor(registry, "ListItemSecondaryAction");
    assert_has_prop(secondary_action, "disableGutters");
    assert_has_slot(secondary_action, "content");

    let text = descriptor(registry, "ListItemText");
    for prop in [
        "primary",
        "secondary",
        "dense",
        "disableTypography",
        "inset",
    ] {
        assert_has_prop(text, prop);
    }
    for slot in ["primary", "secondary"] {
        assert_has_slot(text, slot);
    }

    let subheader = descriptor(registry, "ListSubheader");
    assert_enum_options(subheader, "color", &["default", "inherit", "primary"]);
    for prop in [
        "text",
        "component",
        "disableGutters",
        "disableSticky",
        "inset",
    ] {
        assert_has_prop(subheader, prop);
    }
}

fn assert_image_list_subcomponents(registry: &UiComponentDescriptorRegistry) {
    let item = descriptor(registry, "ImageListItem");
    assert_enum_options(
        item,
        "variant",
        &["masonry", "quilted", "standard", "woven"],
    );
    for prop in ["alt", "cols", "component", "rows", "src"] {
        assert_has_prop(item, prop);
    }
    for slot in ["img", "bar", "content"] {
        assert_has_slot(item, slot);
    }
    assert_default_value(item, "cols", UiValue::Int(1));
    assert_default_value(item, "rows", UiValue::Int(1));
    assert!(item
        .required_render_capabilities
        .contains(&UiRenderCapability::Image));

    let bar = descriptor(registry, "ImageListItemBar");
    assert_enum_options(bar, "actionPosition", &["left", "right"]);
    assert_enum_options(bar, "position", &["below", "bottom", "top"]);
    for prop in ["actionIcon", "subtitle", "title"] {
        assert_has_prop(bar, prop);
    }
    for slot in ["titleWrap", "title", "subtitle", "actionIcon"] {
        assert_has_slot(bar, slot);
    }
}

fn assert_table_subcomponents(registry: &UiComponentDescriptorRegistry) {
    for component_id in ["TableHead", "TableBody", "TableFooter"] {
        let descriptor = descriptor(registry, component_id);
        assert_has_prop(descriptor, "component");
        assert_has_slot(descriptor, "rows");
    }
    assert_has_slot(descriptor(registry, "TableContainer"), "table");

    let row = descriptor(registry, "TableRow");
    for prop in ["component", "hover", "selected", "variant"] {
        assert_has_prop(row, prop);
    }
    assert_enum_options(row, "variant", &["body", "footer", "head"]);
    assert_has_event(row, UiComponentEventKind::SelectOption);

    let cell = descriptor(registry, "TableCell");
    assert_enum_options(
        cell,
        "align",
        &["center", "inherit", "justify", "left", "right"],
    );
    assert_enum_options(cell, "padding", &["checkbox", "none", "normal"]);
    assert_enum_options(cell, "size", &["medium", "small"]);
    assert_enum_options(cell, "variant", &["body", "footer", "head"]);
    for prop in [
        "component",
        "scope",
        "sortDirection",
        "stickyHeader",
        "text",
    ] {
        assert_has_prop(cell, prop);
    }

    let sort_label = descriptor(registry, "TableSortLabel");
    assert_enum_options(sort_label, "direction", &["asc", "desc"]);
    for prop in ["active", "hideSortIcon", "IconComponent", "text"] {
        assert_has_prop(sort_label, prop);
    }
    assert_has_slot(sort_label, "icon");
    assert_has_event(sort_label, UiComponentEventKind::Commit);

    let pagination = descriptor(registry, "TablePagination");
    for prop in [
        "ActionsComponent",
        "component",
        "count",
        "disabled",
        "labelRowsPerPage",
        "page",
        "rowsPerPage",
        "rowsPerPageOptions",
        "showFirstButton",
        "showLastButton",
    ] {
        assert_has_prop(pagination, prop);
    }
    assert_default_value(
        pagination,
        "rowsPerPageOptions",
        UiValue::Array(vec![
            UiValue::Int(10),
            UiValue::Int(25),
            UiValue::Int(50),
            UiValue::Int(100),
        ]),
    );
    for slot in [
        "toolbar",
        "spacer",
        "selectLabel",
        "selectRoot",
        "select",
        "selectIcon",
        "input",
        "menuItem",
        "displayedRows",
        "actions",
    ] {
        assert_has_slot(pagination, slot);
    }
    assert_has_event(pagination, UiComponentEventKind::SetPage);

    let actions = descriptor(registry, "TablePaginationActions");
    for prop in [
        "count",
        "page",
        "rowsPerPage",
        "showFirstButton",
        "showLastButton",
    ] {
        assert_has_prop(actions, prop);
    }
    assert_has_event(actions, UiComponentEventKind::SetPage);
}

fn descriptor<'a>(
    registry: &'a UiComponentDescriptorRegistry,
    id: &str,
) -> &'a zircon_runtime_interface::ui::component::UiComponentDescriptor {
    registry
        .descriptor(id)
        .unwrap_or_else(|| panic!("{id} descriptor"))
}

fn assert_has_slot(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    slot_name: &str,
) {
    assert!(
        descriptor
            .slot_schema
            .iter()
            .any(|slot| slot.name == slot_name),
        "{} missing MUI slot `{slot_name}`",
        descriptor.id
    );
}

fn assert_default_value(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    prop_name: &str,
    expected: UiValue,
) {
    assert_eq!(
        descriptor
            .prop(prop_name)
            .unwrap_or_else(|| panic!("{} missing prop `{prop_name}`", descriptor.id))
            .default_value
            .clone(),
        Some(expected),
        "{} should expose local MUI default for `{prop_name}`",
        descriptor.id
    );
}
