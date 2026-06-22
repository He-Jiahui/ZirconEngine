use super::*;

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
