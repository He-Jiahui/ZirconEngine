use super::*;

#[test]
fn sample_grid_projection_preserves_typed_axes_ticks_and_points() {
    let samples = Value::Array(vec![
        toml_table([
            ("x", Value::Float(0.0)),
            ("y", Value::Float(600.0)),
            ("label", Value::String("Run_Fwd".to_owned())),
            ("selected", Value::Boolean(true)),
        ]),
        toml_table([
            ("x", Value::Float(-180.0)),
            ("y", Value::Float(300.0)),
            ("label", Value::String("Run_Left".to_owned())),
            ("selected", Value::Boolean(false)),
        ]),
    ]);
    let node = host_template_node(projected_node(
        "Canvas",
        [
            ("component_variant", Value::String("sample-grid".to_owned())),
            ("x_axis_label", Value::String("Direction (deg)".to_owned())),
            ("y_axis_label", Value::String("Speed (cm/s)".to_owned())),
            ("x_min", Value::Float(-180.0)),
            ("x_max", Value::Float(180.0)),
            ("y_min", Value::Float(0.0)),
            ("y_max", Value::Float(600.0)),
            (
                "x_ticks",
                Value::Array(
                    [-180.0, -90.0, 0.0, 90.0, 180.0]
                        .into_iter()
                        .map(Value::Float)
                        .collect(),
                ),
            ),
            (
                "y_ticks",
                Value::Array(
                    [0.0, 150.0, 300.0, 450.0, 600.0]
                        .into_iter()
                        .map(Value::Float)
                        .collect(),
                ),
            ),
            ("sample_points", samples),
        ],
    ))
    .expect("sample grid should project");

    let grid = &node.sample_grid.generation;
    assert_eq!(grid.x_axis_label(), "Direction (deg)");
    assert_eq!(grid.y_axis_label(), "Speed (cm/s)");
    assert_eq!(grid.x_min(), -180.0);
    assert_eq!(grid.x_max(), 180.0);
    assert_eq!(grid.y_min(), 0.0);
    assert_eq!(grid.y_max(), 600.0);
    assert_eq!(grid.x_ticks().len(), 5);
    assert_eq!(grid.y_ticks().len(), 5);
    assert_eq!(grid.points().len(), 2);
    let selected = &grid.points()[0];
    assert_eq!(selected.label(), "Run_Fwd");
    assert_eq!((selected.x(), selected.y()), (0.0, 600.0));
    assert!(selected.selected());
}

#[test]
fn malformed_sample_grid_entries_are_ignored_without_losing_valid_points() {
    let node = host_template_node(projected_node(
        "Canvas",
        [
            ("component_variant", Value::String("sample-grid".to_owned())),
            (
                "sample_points",
                Value::Array(vec![
                    Value::String("not-a-point".to_owned()),
                    toml_table([
                        ("x", Value::Float(12.0)),
                        ("y", Value::Float(34.0)),
                        ("label", Value::String("Walk".to_owned())),
                    ]),
                ]),
            ),
        ],
    ))
    .expect("sample grid should project");

    let points = node.sample_grid.generation.points();
    assert_eq!(points.len(), 1);
    assert_eq!(points[0].label(), "Walk");
}
