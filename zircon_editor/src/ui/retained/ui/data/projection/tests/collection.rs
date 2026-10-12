use super::*;

fn table_row(surface_entity: i64, label: &str) -> Value {
    let mut row = toml::map::Map::new();
    row.insert("surface_entity".to_string(), Value::Integer(surface_entity));
    row.insert("label".to_string(), Value::String(label.to_string()));
    Value::Table(row)
}

#[test]
fn runtime_component_projection_preserves_virtualization_and_pagination_metadata() {
    let virtual_list = host_template_node(projected_node(
        "VirtualList",
        [
            ("item_extent", Value::Float(32.0)),
            ("overscan", Value::Integer(4)),
            ("total_count", Value::Integer(1000)),
            ("viewport_start", Value::Integer(120)),
            ("viewport_count", Value::Integer(40)),
        ],
    ))
    .expect("VirtualList should project into the host contract");

    assert!(virtual_list.virtualization_enabled);
    assert_eq!(virtual_list.virtualization_item_extent, 32.0);
    assert_eq!(virtual_list.virtualization_overscan, 4);
    assert_eq!(virtual_list.virtualization_total_count, 1000);
    assert_eq!(virtual_list.virtualization_visible_start, 120);
    assert_eq!(virtual_list.virtualization_visible_count, 40);
    assert_eq!(virtual_list.component_category.as_str(), "collection");
    assert_eq!(virtual_list.component_layout_role.as_str(), "virtual-list");

    let paged_list = host_template_node(projected_node(
        "PagedList",
        [
            ("total_count", Value::Integer(2500)),
            ("page_index", Value::Integer(3)),
            ("page_size", Value::Integer(100)),
            ("page_count", Value::Integer(25)),
        ],
    ))
    .expect("PagedList should project into the host contract");

    assert!(!paged_list.virtualization_enabled);
    assert_eq!(paged_list.pagination_total_count, 2500);
    assert_eq!(paged_list.pagination_page_index, 3);
    assert_eq!(paged_list.pagination_page_size, 100);
    assert_eq!(paged_list.pagination_page_count, 25);
}

#[test]
fn runtime_component_projection_slices_virtualized_visible_collection_items() {
    let virtual_list = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..20).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(10)),
            ("viewport_count", Value::Integer(5)),
            ("overscan", Value::Integer(2)),
        ],
    ))
    .expect("VirtualList should project a visible collection window");

    assert_eq!(virtual_list.collection_items.row_count(), 9);
    assert_eq!(
        virtual_list.collection_items.row_data(0).as_deref(),
        Some("Item 8")
    );
    assert_eq!(
        virtual_list.collection_items.row_data(8).as_deref(),
        Some("Item 16")
    );
}

#[test]
fn runtime_component_projection_clamps_virtualized_collection_window_edges() {
    let negative_start = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..5).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(-10)),
            ("viewport_count", Value::Integer(2)),
            ("overscan", Value::Integer(1)),
        ],
    ))
    .expect("VirtualList should project a negative start deterministically");

    assert_eq!(negative_start.collection_items.row_count(), 3);
    assert_eq!(
        negative_start.collection_items.row_data(0).as_deref(),
        Some("Item 0")
    );
    assert_eq!(
        negative_start.collection_items.row_data(2).as_deref(),
        Some("Item 2")
    );

    let zero_count = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..5).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(1)),
            ("viewport_count", Value::Integer(0)),
            ("overscan", Value::Integer(10)),
        ],
    ))
    .expect("VirtualList should project a zero visible count deterministically");

    assert_eq!(zero_count.collection_items.row_count(), 0);

    let oversized_overscan = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..4).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(2)),
            ("viewport_count", Value::Integer(1)),
            ("overscan", Value::Integer(50)),
        ],
    ))
    .expect("VirtualList should project oversized overscan deterministically");

    assert_eq!(oversized_overscan.collection_items.row_count(), 4);
}

#[test]
fn runtime_table_projection_preserves_typed_identity_and_source_index() {
    let table = host_template_node(projected_node(
        "Table",
        [
            (
                "row_identity_field",
                Value::String("surface_entity".to_string()),
            ),
            (
                "rows",
                Value::Array(vec![table_row(41, "Ground"), table_row(73, "Roof")]),
            ),
        ],
    ))
    .expect("Table should project typed row identity data");

    assert_eq!(table.collection_rows.row_count(), 2);
    let first = table
        .collection_rows
        .row_data(0)
        .expect("first table row should be projected");
    assert_eq!(first.source_index, 0);
    assert_eq!(first.row_identity_field.as_str(), "surface_entity");
    assert_eq!(first.identity_kind.as_str(), "integer");
    assert_eq!(first.identity_text.as_str(), "41");
    assert_eq!(first.label.as_str(), "Ground");

    let second = table
        .collection_rows
        .row_data(1)
        .expect("second table row should be projected");
    assert_eq!(second.source_index, 1);
    assert_eq!(second.identity_text.as_str(), "73");
}

#[test]
fn runtime_table_projection_uses_identity_as_the_default_label() {
    let mut row = toml::map::Map::new();
    row.insert("surface_entity".to_string(), Value::Integer(91));
    let table = host_template_node(projected_node(
        "Table",
        [
            (
                "row_identity_field",
                Value::String("surface_entity".to_string()),
            ),
            ("rows", Value::Array(vec![Value::Table(row)])),
        ],
    ))
    .expect("Table should derive a label from its row identity");

    let row = table
        .collection_rows
        .row_data(0)
        .expect("table row should be projected");
    assert_eq!(row.identity_text.as_str(), "91");
    assert_eq!(row.label.as_str(), "91");
}
