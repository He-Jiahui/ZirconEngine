use super::*;

#[test]
fn weight_heatmap_projection_preserves_typed_resolution_labels_and_sources() {
    let node = host_template_node(projected_node(
        "Canvas",
        [
            (
                "component_variant",
                Value::String("weight-heatmap".to_owned()),
            ),
            ("heatmap_columns", Value::Integer(16)),
            ("heatmap_rows", Value::Integer(10)),
            ("low_label", Value::String("Cold".to_owned())),
            ("high_label", Value::String("Hot".to_owned())),
            (
                "heat_sources",
                Value::Array(vec![
                    toml_table([
                        ("x", Value::Float(0.5)),
                        ("y", Value::Float(0.6)),
                        ("weight", Value::Float(1.0)),
                        ("selected", Value::Boolean(true)),
                    ]),
                    toml_table([
                        ("x", Value::Float(0.1)),
                        ("y", Value::Float(0.2)),
                        ("weight", Value::Float(0.35)),
                        ("selected", Value::Boolean(false)),
                    ]),
                ]),
            ),
        ],
    ))
    .expect("weight heatmap should project");

    assert_eq!(
        (
            node.weight_heatmap.generation.columns(),
            node.weight_heatmap.generation.rows(),
        ),
        (16, 10)
    );
    assert_eq!(node.weight_heatmap.generation.low_label(), "Cold");
    assert_eq!(node.weight_heatmap.generation.high_label(), "Hot");
    assert_eq!(node.weight_heatmap.generation.sources().len(), 2);
    let selected = node
        .weight_heatmap
        .generation
        .sources()
        .first()
        .expect("selected source");
    assert_eq!(
        (selected.x(), selected.y(), selected.weight()),
        (0.5, 0.6, 1.0)
    );
    assert!(selected.selected());
}

#[test]
fn weight_heatmap_projection_clamps_resolution_and_normalized_source_values() {
    let node = host_template_node(projected_node(
        "Canvas",
        [
            (
                "component_variant",
                Value::String("weight-heatmap".to_owned()),
            ),
            ("heatmap_columns", Value::Integer(200)),
            ("heatmap_rows", Value::Integer(1)),
            (
                "heat_sources",
                Value::Array(vec![
                    Value::String("invalid".to_owned()),
                    toml_table([
                        ("x", Value::Float(-2.0)),
                        ("y", Value::Float(4.0)),
                        ("weight", Value::Float(3.0)),
                    ]),
                ]),
            ),
        ],
    ))
    .expect("weight heatmap should project");

    assert_eq!(
        (
            node.weight_heatmap.generation.columns(),
            node.weight_heatmap.generation.rows(),
        ),
        (32, 3)
    );
    assert_eq!(node.weight_heatmap.generation.sources().len(), 1);
    let source = node
        .weight_heatmap
        .generation
        .sources()
        .first()
        .expect("valid source");
    assert_eq!((source.x(), source.y(), source.weight()), (0.0, 1.0, 1.0));
}
