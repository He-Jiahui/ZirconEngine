use super::*;

#[test]
fn timeline_strip_projection_preserves_duration_playhead_track_and_keys() {
    let node = host_template_node(projected_node(
        "Canvas",
        [
            (
                "component_variant",
                Value::String("timeline-strip".to_owned()),
            ),
            ("duration", Value::Float(3.0)),
            ("current_time", Value::Float(2.25)),
            ("tick_interval", Value::Float(0.5)),
            ("track_label", Value::String("Run_Fwd".to_owned())),
            (
                "timeline_keys",
                Value::Array(vec![
                    toml_table([
                        ("time", Value::Float(0.0)),
                        ("label", Value::String("Start".to_owned())),
                        ("selected", Value::Boolean(false)),
                    ]),
                    toml_table([
                        ("time", Value::Float(2.0)),
                        ("label", Value::String("Run_Fwd".to_owned())),
                        ("selected", Value::Boolean(true)),
                    ]),
                ]),
            ),
        ],
    ))
    .expect("timeline strip should project");

    let generation = &node.timeline_strip.generation;
    assert_eq!(generation.duration(), 3.0);
    assert_eq!(generation.current_time(), 2.25);
    assert_eq!(generation.tick_interval(), 0.5);
    assert_eq!(generation.track_label(), "Run_Fwd");
    assert_eq!(generation.keys().len(), 2);
    let selected = &generation.keys()[1];
    assert_eq!(selected.time(), 2.0);
    assert_eq!(selected.label(), "Run_Fwd");
    assert!(selected.selected());
}

#[test]
fn timeline_strip_projection_normalizes_ranges_and_filters_malformed_keys() {
    let node = host_template_node(projected_node(
        "Canvas",
        [
            (
                "component_variant",
                Value::String("timeline-strip".to_owned()),
            ),
            ("duration", Value::Float(-4.0)),
            ("current_time", Value::Float(99.0)),
            ("tick_interval", Value::Float(0.0)),
            (
                "timeline_keys",
                Value::Array(vec![
                    Value::String("invalid".to_owned()),
                    toml_table([("time", Value::Float(8.0))]),
                ]),
            ),
        ],
    ))
    .expect("timeline strip should project");

    let generation = &node.timeline_strip.generation;
    assert_eq!(generation.duration(), 1.0);
    assert_eq!(generation.current_time(), 1.0);
    assert_eq!(generation.tick_interval(), 0.25);
    assert_eq!(generation.keys().len(), 1);
    assert_eq!(generation.keys()[0].time(), 1.0);
}
