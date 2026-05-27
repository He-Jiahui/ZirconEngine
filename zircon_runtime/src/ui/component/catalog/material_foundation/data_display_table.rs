use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const TABLE_CELL_ALIGNS: [&str; 5] = ["center", "inherit", "justify", "left", "right"];
const TABLE_CELL_PADDINGS: [&str; 3] = ["checkbox", "none", "normal"];
const TABLE_CELL_VARIANTS: [&str; 3] = ["body", "footer", "head"];
const TABLE_SIZES: [&str; 2] = ["medium", "small"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        table_container(),
        table_group("TableHead", "Table Head", "thead"),
        table_group("TableBody", "Table Body", "tbody"),
        table_group("TableFooter", "Table Footer", "tfoot"),
        table_row(),
        table_cell(),
        table_sort_label(),
        table_pagination(),
        table_pagination_actions(),
    ]
}

fn table_container() -> UiComponentDescriptor {
    composite(
        "TableContainer",
        "Table Container",
        UiComponentCategory::Container,
        "table-container",
    )
    .with_prop(default_string_prop("component", "div"))
    .slot(UiSlotSchema::new("table"))
    .requires_render_capability(UiRenderCapability::Scroll)
}

fn table_group(id: &str, display_name: &str, component: &str) -> UiComponentDescriptor {
    composite(id, display_name, UiComponentCategory::Collection, component)
        .with_prop(default_string_prop("component", component))
        .slot(UiSlotSchema::new("rows").multiple(true))
}

fn table_row() -> UiComponentDescriptor {
    composite(
        "TableRow",
        "Table Row",
        UiComponentCategory::Collection,
        "table-row",
    )
    .with_prop(default_string_prop("component", "tr"))
    .with_prop(bool_prop("hover", false))
    .with_prop(bool_prop("selected", false))
    .with_prop(mui_enum_prop("variant", "body", TABLE_CELL_VARIANTS))
    .slot(UiSlotSchema::new("cells").multiple(true))
    .events([
        UiComponentEventKind::Hover,
        UiComponentEventKind::SelectOption,
    ])
}

fn table_cell() -> UiComponentDescriptor {
    primitive(
        "TableCell",
        "Table Cell",
        UiComponentCategory::Collection,
        "table-cell",
    )
    .with_prop(text_prop())
    .with_prop(default_string_prop("component", ""))
    .with_prop(mui_enum_prop("align", "inherit", TABLE_CELL_ALIGNS))
    .with_prop(mui_enum_prop("padding", "normal", TABLE_CELL_PADDINGS))
    .with_prop(default_string_prop("scope", ""))
    .with_prop(mui_enum_prop("size", "medium", TABLE_SIZES))
    .with_prop(mui_enum_prop(
        "sortDirection",
        "none",
        ["asc", "desc", "none"],
    ))
    .with_prop(bool_prop("stickyHeader", false))
    .with_prop(mui_enum_prop("variant", "body", TABLE_CELL_VARIANTS))
}

fn table_sort_label() -> UiComponentDescriptor {
    primitive(
        "TableSortLabel",
        "Table Sort Label",
        UiComponentCategory::Input,
        "table-sort-label",
    )
    .with_prop(text_prop())
    .with_prop(bool_prop("active", false))
    .with_prop(mui_enum_prop("direction", "asc", ["asc", "desc"]))
    .with_prop(bool_prop("hideSortIcon", false))
    .with_prop(default_string_prop("IconComponent", "ArrowDownwardIcon"))
    .slot(UiSlotSchema::new("icon"))
    .events([
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Commit,
    ])
}

fn table_pagination() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "TablePagination",
                "Table Pagination",
                UiComponentCategory::Collection,
                "table-pagination",
            ),
            [
                default_string_prop("ActionsComponent", "TablePaginationActions"),
                default_string_prop("component", "TableCell"),
                int_prop("count", 0),
                bool_prop("disabled", false),
                default_string_prop("labelRowsPerPage", "Rows per page:"),
                int_prop("page", 0),
                int_prop("rowsPerPage", 10),
                rows_per_page_options_prop(),
                bool_prop("showFirstButton", false),
                bool_prop("showLastButton", false),
            ],
        ),
        [
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
        ],
    )
    .events([
        UiComponentEventKind::SetPage,
        UiComponentEventKind::ValueChanged,
    ])
}

fn table_pagination_actions() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "TablePaginationActions",
                "Table Pagination Actions",
                UiComponentCategory::Input,
                "table-pagination-actions",
            ),
            [
                int_prop("count", 0),
                bool_prop("disabled", false),
                default_string_prop("getItemAriaLabel", ""),
                int_prop("page", 0),
                int_prop("rowsPerPage", 10),
                bool_prop("showFirstButton", false),
                bool_prop("showLastButton", false),
            ],
        ),
        [
            "firstButton",
            "firstButtonIcon",
            "lastButton",
            "lastButtonIcon",
            "nextButton",
            "nextButtonIcon",
            "previousButton",
            "previousButtonIcon",
        ],
    )
    .event(UiComponentEventKind::SetPage)
}

fn add_props<const N: usize>(
    mut descriptor: UiComponentDescriptor,
    props: [UiPropSchema; N],
) -> UiComponentDescriptor {
    for prop in props {
        descriptor = descriptor.with_prop(prop);
    }
    descriptor
}

fn add_slots<const N: usize>(
    mut descriptor: UiComponentDescriptor,
    names: [&str; N],
) -> UiComponentDescriptor {
    for name in names {
        descriptor = descriptor.slot(UiSlotSchema::new(name));
    }
    descriptor
}

fn mui_enum_prop<const N: usize>(
    name: &str,
    default: &str,
    options: [&'static str; N],
) -> UiPropSchema {
    enum_prop_with_options(
        name,
        default,
        options.into_iter().map(enum_option_descriptor),
    )
}

fn rows_per_page_options_prop() -> UiPropSchema {
    UiPropSchema::new("rowsPerPageOptions", UiValueKind::Array).default_value(UiValue::Array(vec![
        UiValue::Int(10),
        UiValue::Int(25),
        UiValue::Int(50),
        UiValue::Int(100),
    ]))
}
