use super::*;
use std::collections::BTreeMap;

#[test]
fn table_column_resize_drag_updates_widths_and_emits_value_changed() {
    let mut surface = table_pointer_route_surface(false, false);

    let begin = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(168.0, 22.0),
    );
    assert!(begin.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::BeginDrag {
                    property: "column_width".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::Begin)
    }));
    assert!(begin.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::CapturePointer { target, .. } if *target == UiNodeId::new(2)
    )));

    let update = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(196.0, 22.0),
    );
    assert!(update.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "column_width".to_string(),
                    value: column_width_payload("triangles", 124.0),
                }
    }));
    assert!(update.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::DragDelta {
                    property: "column_width".to_string(),
                    delta: 28.0,
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::Update)
    }));
    assert_table_column_width(&surface, "triangles", 124.0);

    let end = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(196.0, 22.0),
    );
    assert!(end.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::EndDrag {
                    property: "column_width".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::End)
    }));
    assert!(end.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::ReleasePointerCapture { target, .. } if *target == UiNodeId::new(2)
    )));
    assert_table_column_width(&surface, "triangles", 124.0);
}

#[test]
fn data_grid_column_resize_drag_updates_mui_grid_widths() {
    let mut surface = table_pointer_route_surface(true, false);

    dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(168.0, 22.0),
    );
    let update = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(200.0, 22.0),
    );

    assert!(update.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "column_width".to_string(),
                    value: column_width_payload("triangles", 128.0),
                }
    }));
    assert_table_column_width(&surface, "triangles", 128.0);
}

#[test]
fn data_grid_disable_column_resize_blocks_default_resize_drag() {
    let mut surface = table_pointer_route_surface(true, true);

    let begin = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(168.0, 22.0),
    );
    dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(210.0, 22.0),
    );
    let end = dispatch_table_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(210.0, 22.0),
    );

    assert!(!begin.component_events.iter().any(|event| {
        matches!(event.event, UiComponentEvent::BeginDrag { .. })
            || matches!(event.event, UiComponentEvent::ValueChanged { .. })
    }));
    assert!(!end
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::EndDrag { .. })));
    assert_table_column_width(&surface, "triangles", 96.0);
}

#[test]
fn table_sort_header_click_toggles_direction_and_sorts_rows() {
    let mut surface = table_pointer_route_surface(false, false);

    let first = click_table_pointer(&mut surface, UiPoint::new(190.0, 22.0));

    assert!(first.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "sort_column".to_string(),
                    value: UiValue::String("triangles".to_string()),
                }
    }));
    assert_table_sort_state(&surface, "triangles", "asc");
    assert_table_column_sort_indicator(&surface, "triangles", "asc");
    assert_table_row_ids(&surface, ["camera", "cube", "sphere"]);

    click_table_pointer(&mut surface, UiPoint::new(190.0, 22.0));

    assert_table_sort_state(&surface, "triangles", "desc");
    assert_table_column_sort_indicator(&surface, "triangles", "desc");
    assert_table_row_ids(&surface, ["sphere", "cube", "camera"]);
}

#[test]
fn data_grid_server_sort_header_click_updates_sort_model_without_reordering_rows() {
    let mut surface = table_pointer_route_surface_with_options(true, false, true);

    let result = click_table_pointer(&mut surface, UiPoint::new(190.0, 22.0));

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "sort_column".to_string(),
                    value: UiValue::String("triangles".to_string()),
                }
    }));
    assert_table_sort_state(&surface, "triangles", "asc");
    assert_table_sort_model(&surface, "triangles", "asc");
    assert_table_row_ids(&surface, ["sphere", "cube", "camera"]);
}

#[test]
fn table_row_click_selects_row_on_owner() {
    let mut surface = table_pointer_route_surface(false, false);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 76.0));

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selected_rows".to_string(),
                    option_id: "cube".to_string(),
                    selected: true,
                }
    }));
    assert_table_attr_strings(&surface, "selected_rows", &["cube"]);
    assert_table_attr_string(&surface, "value", "cube");
    assert_table_attr_int(&surface, "focused_index", 1);
    assert_table_attr_int(&surface, "selected_index", 1);
}

#[test]
fn data_grid_row_click_updates_row_selection_model() {
    let mut surface = table_pointer_route_surface(true, false);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 100.0));

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "rowSelectionModel".to_string(),
                    option_id: "camera".to_string(),
                    selected: true,
                }
    }));
    assert_table_attr_strings(&surface, "rowSelectionModel", &["camera"]);
    assert_table_attr_int(&surface, "focused_index", 2);
    assert_table_attr_int(&surface, "selected_index", 2);
}

#[test]
fn data_grid_disable_row_selection_on_click_blocks_row_selection() {
    let mut surface =
        table_pointer_route_surface_with_row_selection_options(true, false, false, true);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 100.0));

    assert!(!result
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::SelectOption { .. })));
    assert_table_attr_strings(&surface, "rowSelectionModel", &[]);
    assert_table_attr_missing(&surface, "focused_index");
    assert_table_attr_missing(&surface, "selected_index");
}

#[test]
fn table_scroll_updates_virtual_window_and_emits_visible_range() {
    let mut surface = table_pointer_route_surface(false, false);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 76.0), 48.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 4 }
    }));
    assert_table_attr_int(&surface, "viewport_start", 2);
    assert_table_attr_int(&surface, "viewport_count", 4);
    assert_table_attr_int(&surface, "visible_end", 6);
    assert_table_attr_int(&surface, "requested_start", 0);
    assert_table_attr_int(&surface, "requested_count", 8);
    assert_table_attr_float(&surface, "scrollTop", 48.0);
}

#[test]
fn data_grid_scroll_updates_mui_virtual_window_aliases() {
    let mut surface = table_pointer_route_surface(true, false);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 100.0), 72.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 3, count: 4 }
    }));
    assert_table_attr_int(&surface, "viewport_start", 3);
    assert_table_attr_int(&surface, "viewport_count", 4);
    assert_table_attr_int(&surface, "visibleEnd", 7);
    assert_table_attr_int(&surface, "requestedStart", 1);
    assert_table_attr_int(&surface, "requestedCount", 8);
    assert_table_attr_float(&surface, "scrollTop", 72.0);
}

#[test]
fn data_grid_disable_virtualization_blocks_default_virtual_scroll() {
    let mut surface =
        table_pointer_route_surface_with_virtualization_options(true, false, false, false, true);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 100.0), 72.0);

    assert!(!result
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::SetVisibleRange { .. })));
    assert_table_attr_int(&surface, "viewport_start", 0);
    assert_table_attr_float(&surface, "scrollTop", 0.0);
}

fn dispatch_table_pointer(
    surface: &mut UiSurface,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            table_pointer_event(kind, point),
        )
        .expect("table pointer event should dispatch")
}

fn dispatch_table_scroll(
    surface: &mut UiSurface,
    point: UiPoint,
    scroll_delta: f32,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            table_scroll_event(point, scroll_delta),
        )
        .expect("table scroll event should dispatch")
}

fn table_pointer_event(kind: UiPointerEventKind, point: UiPoint) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(kind, point)
            .with_button(UiPointerButton::Primary)
            .with_click_count(1),
        precise_scroll: None,
    })
}

fn table_scroll_event(point: UiPoint, scroll_delta: f32) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(UiPointerEventKind::Scroll, point)
            .with_scroll_delta(scroll_delta),
        precise_scroll: None,
    })
}

fn click_table_pointer(surface: &mut UiSurface, point: UiPoint) -> UiInputDispatchResult {
    dispatch_table_pointer(surface, UiPointerEventKind::Down, point);
    dispatch_table_pointer(surface, UiPointerEventKind::Up, point)
}

fn table_pointer_route_surface(data_grid: bool, disable_resize: bool) -> UiSurface {
    table_pointer_route_surface_with_options(data_grid, disable_resize, false)
}

fn table_pointer_route_surface_with_options(
    data_grid: bool,
    disable_resize: bool,
    server_sort: bool,
) -> UiSurface {
    table_pointer_route_surface_with_row_selection_options(
        data_grid,
        disable_resize,
        server_sort,
        false,
    )
}

fn table_pointer_route_surface_with_row_selection_options(
    data_grid: bool,
    disable_resize: bool,
    server_sort: bool,
    disable_row_selection_on_click: bool,
) -> UiSurface {
    table_pointer_route_surface_with_virtualization_options(
        data_grid,
        disable_resize,
        server_sort,
        disable_row_selection_on_click,
        false,
    )
}

fn table_pointer_route_surface_with_virtualization_options(
    data_grid: bool,
    disable_resize: bool,
    server_sort: bool,
    disable_row_selection_on_click: bool,
    disable_virtualization: bool,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.table"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 300.0, 160.0))
            .with_state_flags(input_state()),
    );

    let component = if data_grid { "DataGrid" } else { "Table" };
    let disable_column_resize = if disable_resize {
        "disableColumnResize = true"
    } else {
        ""
    };
    let sorting_mode = if server_sort {
        r#"sortingMode = "server""#
    } else {
        ""
    };
    let sort_model = if data_grid { "sortModel = []" } else { "" };
    let row_selection_model = if data_grid {
        "rowSelectionModel = []"
    } else {
        ""
    };
    let disable_row_selection = if disable_row_selection_on_click {
        "disableRowSelectionOnClick = true"
    } else {
        ""
    };
    let disable_virtualization = if disable_virtualization {
        "disableVirtualization = true"
    } else {
        ""
    };
    let table_attributes: BTreeMap<String, toml::Value> = toml::from_str(&format!(
        r#"
columns = [
    {{ field = "name", width = 160.0 }},
    {{ field = "triangles", width = 96.0 }},
]
rows = [
    {{ id = "sphere", name = "Sphere", triangles = 64 }},
    {{ id = "cube", name = "Cube", triangles = 12 }},
    {{ id = "camera", name = "Camera", triangles = 1 }},
]
row_count = 40
rowCount = 40
rowHeight = 24.0
overscanCount = 2
viewport_start = 0
viewport_count = 4
scrollTop = 0.0
column_widths = {{ name = 160.0, triangles = 96.0 }}
sort_column = ""
sort_direction = "none"
selected_rows = []
value = ""
{sort_model}
{row_selection_model}
resizable_columns = true
min_column_width = 40.0
{disable_column_resize}
{sorting_mode}
{disable_row_selection}
{disable_virtualization}
"#
    ))
    .expect("table attributes should parse");

    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/table"))
                .with_frame(UiFrame::new(10.0, 10.0, 260.0, 100.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    control_id: Some("GeometryTable".to_string()),
                    attributes: table_attributes,
                    bindings: vec![
                        binding("GeometryTable/ValueChanged", UiEventKind::Change),
                        binding("GeometryTable/BeginDrag", UiEventKind::DragBegin),
                        binding("GeometryTable/DragDelta", UiEventKind::DragUpdate),
                        binding("GeometryTable/EndDrag", UiEventKind::DragEnd),
                        binding("GeometryTable/SelectOption", UiEventKind::Change),
                        binding("GeometryTable/SetVisibleRange", UiEventKind::Change),
                    ],
                    ..Default::default()
                }),
        )
        .unwrap();

    insert_table_sort_header(
        &mut surface,
        data_grid,
        UiNodeId::new(4),
        "root/table/name-header",
        "name",
        UiFrame::new(10.0, 10.0, 150.0, 28.0),
    );
    insert_table_sort_header(
        &mut surface,
        data_grid,
        UiNodeId::new(5),
        "root/table/triangles-header",
        "triangles",
        UiFrame::new(172.0, 10.0, 90.0, 28.0),
    );

    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(
                UiNodeId::new(3),
                UiNodePath::new("root/table/triangles-resize"),
            )
            .with_frame(UiFrame::new(164.0, 10.0, 8.0, 28.0))
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(input_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: if data_grid {
                    "DataGridColumnResizeHandle".to_string()
                } else {
                    "TableColumnResizeHandle".to_string()
                },
                control_id: Some("TrianglesResize".to_string()),
                attributes: toml::from_str(r#"field = "triangles""#)
                    .expect("resize handle attributes should parse"),
                ..Default::default()
            }),
        )
        .unwrap();

    insert_table_row(
        &mut surface,
        data_grid,
        UiNodeId::new(6),
        "root/table/sphere-row",
        "sphere",
        0,
        UiFrame::new(10.0, 42.0, 252.0, 24.0),
    );
    insert_table_row(
        &mut surface,
        data_grid,
        UiNodeId::new(7),
        "root/table/cube-row",
        "cube",
        1,
        UiFrame::new(10.0, 66.0, 252.0, 24.0),
    );
    insert_table_row(
        &mut surface,
        data_grid,
        UiNodeId::new(8),
        "root/table/camera-row",
        "camera",
        2,
        UiFrame::new(10.0, 90.0, 252.0, 24.0),
    );

    surface.rebuild();
    surface
}

fn insert_table_sort_header(
    surface: &mut UiSurface,
    data_grid: bool,
    node_id: UiNodeId,
    path: &str,
    field: &str,
    frame: UiFrame,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(node_id, UiNodePath::new(path))
                .with_frame(frame)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: if data_grid {
                        "DataGridColumnHeader".to_string()
                    } else {
                        "TableColumnHeader".to_string()
                    },
                    control_id: Some(format!("{field}Header")),
                    attributes: toml::from_str(&format!(r#"field = "{field}""#))
                        .expect("sort header attributes should parse"),
                    ..Default::default()
                }),
        )
        .unwrap();
}

fn insert_table_row(
    surface: &mut UiSurface,
    data_grid: bool,
    node_id: UiNodeId,
    path: &str,
    row_id: &str,
    row_index: usize,
    frame: UiFrame,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(node_id, UiNodePath::new(path))
                .with_frame(frame)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: if data_grid {
                        "DataGridRow".to_string()
                    } else {
                        "TableRow".to_string()
                    },
                    control_id: Some(format!("{row_id}Row")),
                    attributes: toml::from_str(&format!(
                        r#"
row_id = "{row_id}"
row_index = {row_index}
"#
                    ))
                    .expect("row attributes should parse"),
                    ..Default::default()
                }),
        )
        .unwrap();
}

fn assert_table_column_width(surface: &UiSurface, field: &str, expected: f64) {
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("table metadata should exist");
    let width = metadata
        .attributes
        .get("column_widths")
        .and_then(toml::Value::as_table)
        .and_then(|widths| widths.get(field))
        .and_then(toml_number);
    assert_eq!(width, Some(expected));

    let column_width = metadata
        .attributes
        .get("columns")
        .and_then(toml::Value::as_array)
        .and_then(|columns| {
            columns.iter().find_map(|column| {
                let values = column.as_table()?;
                let column_field = values.get("field")?.as_str()?;
                (column_field == field).then(|| values.get("width").and_then(toml_number))?
            })
        });
    assert_eq!(column_width, Some(expected));
}

fn toml_number(value: &toml::Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
}

fn column_width_payload(field: &str, width: f64) -> UiValue {
    UiValue::Map(BTreeMap::from([
        ("field".to_string(), UiValue::String(field.to_string())),
        ("width".to_string(), UiValue::Float(width)),
    ]))
}

fn assert_table_sort_state(surface: &UiSurface, field: &str, direction: &str) {
    let metadata = table_metadata(surface);
    assert_eq!(
        metadata
            .attributes
            .get("sort_column")
            .and_then(toml::Value::as_str),
        Some(field)
    );
    assert_eq!(
        metadata
            .attributes
            .get("sort_direction")
            .and_then(toml::Value::as_str),
        Some(direction)
    );
}

fn assert_table_column_sort_indicator(surface: &UiSurface, field: &str, direction: &str) {
    let metadata = table_metadata(surface);
    let indicator = metadata
        .attributes
        .get("columns")
        .and_then(toml::Value::as_array)
        .and_then(|columns| {
            columns.iter().find_map(|column| {
                let values = column.as_table()?;
                let column_field = values.get("field")?.as_str()?;
                (column_field == field).then(|| values.get("sortDirection")?.as_str())?
            })
        });
    assert_eq!(indicator, Some(direction));
}

fn assert_table_sort_model(surface: &UiSurface, field: &str, direction: &str) {
    let metadata = table_metadata(surface);
    let sort_model = metadata
        .attributes
        .get("sortModel")
        .and_then(toml::Value::as_array)
        .and_then(|entries| entries.first())
        .and_then(toml::Value::as_table)
        .expect("sortModel should contain an entry");
    assert_eq!(
        sort_model.get("field").and_then(toml::Value::as_str),
        Some(field)
    );
    assert_eq!(
        sort_model.get("sort").and_then(toml::Value::as_str),
        Some(direction)
    );
}

fn assert_table_row_ids<const N: usize>(surface: &UiSurface, expected: [&str; N]) {
    let metadata = table_metadata(surface);
    let rows = metadata
        .attributes
        .get("rows")
        .and_then(toml::Value::as_array)
        .expect("rows should exist");
    let actual: Vec<_> = rows
        .iter()
        .map(|row| {
            row.as_table()
                .and_then(|row| row.get("id"))
                .and_then(toml::Value::as_str)
                .expect("row id should exist")
        })
        .collect();
    assert_eq!(actual, expected);
}

fn assert_table_attr_strings(surface: &UiSurface, property: &str, expected: &[&str]) {
    let metadata = table_metadata(surface);
    let values = metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_array)
        .expect("table string array attribute should exist");
    let actual: Vec<_> = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("table string array item should exist")
        })
        .collect();
    assert_eq!(actual, expected);
}

fn assert_table_attr_string(surface: &UiSurface, property: &str, expected: &str) {
    let metadata = table_metadata(surface);
    assert_eq!(
        metadata
            .attributes
            .get(property)
            .and_then(toml::Value::as_str),
        Some(expected)
    );
}

fn assert_table_attr_int(surface: &UiSurface, property: &str, expected: i64) {
    let metadata = table_metadata(surface);
    assert_eq!(
        metadata
            .attributes
            .get(property)
            .and_then(toml::Value::as_integer),
        Some(expected)
    );
}

fn assert_table_attr_float(surface: &UiSurface, property: &str, expected: f64) {
    let metadata = table_metadata(surface);
    assert_eq!(
        metadata.attributes.get(property).and_then(toml_number),
        Some(expected)
    );
}

fn assert_table_attr_missing(surface: &UiSurface, property: &str) {
    let metadata = table_metadata(surface);
    assert!(!metadata.attributes.contains_key(property));
}

fn table_metadata(surface: &UiSurface) -> &UiTemplateNodeMetadata {
    surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("table metadata should exist")
}
