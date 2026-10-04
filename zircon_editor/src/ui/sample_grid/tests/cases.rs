use super::{SampleGridGeneration, SampleGridGenerationInput, SampleGridPoint};

fn generation(
    x_axis_label: &str,
    x_ticks: Vec<f32>,
    point: SampleGridPoint,
) -> SampleGridGeneration {
    generation_with_ranges(x_axis_label, x_ticks, point, [-180.0, 180.0, 0.0, 600.0])
}

fn generation_with_ranges(
    x_axis_label: &str,
    x_ticks: Vec<f32>,
    point: SampleGridPoint,
    [x_min, x_max, y_min, y_max]: [f32; 4],
) -> SampleGridGeneration {
    SampleGridGeneration::new(SampleGridGenerationInput {
        x_axis_label: x_axis_label.to_string(),
        y_axis_label: "Speed".to_string(),
        x_min,
        x_max,
        y_min,
        y_max,
        x_ticks,
        y_ticks: vec![0.0, 300.0, 600.0],
        points: vec![point],
    })
}

#[test]
fn ticks_are_preformatted_once_in_generation() {
    let generation = generation(
        "Direction",
        vec![-180.0, -22.25, 0.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", false),
    );

    let labels = generation
        .x_ticks()
        .iter()
        .map(|tick| tick.label())
        .collect::<Vec<_>>();
    assert_eq!(labels, vec!["-180", "-22.2", "0"]);
}

#[test]
fn selection_changes_only_dynamic_generation() {
    let idle = generation(
        "Direction",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", false),
    );
    let selected = generation(
        "Direction",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", true),
    );

    assert_eq!(idle.static_generation(), selected.static_generation());
    assert_ne!(idle.dynamic_generation(), selected.dynamic_generation());
}

#[test]
fn point_drag_changes_only_dynamic_generation() {
    let before = generation(
        "Direction",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", true),
    );
    let after = generation(
        "Direction",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(30.0, 420.0, "Walk", true),
    );

    assert_eq!(before.static_generation(), after.static_generation());
    assert_ne!(before.dynamic_generation(), after.dynamic_generation());
}

#[test]
fn axis_and_tick_changes_update_static_generation() {
    let baseline = generation(
        "Direction",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", false),
    );
    let relabeled = generation(
        "Heading",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", false),
    );
    let reticked = generation(
        "Direction",
        vec![-180.0, -90.0, 0.0, 90.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", false),
    );

    assert_ne!(baseline.static_generation(), relabeled.static_generation());
    assert_ne!(baseline.static_generation(), reticked.static_generation());
    assert_eq!(
        baseline.dynamic_generation(),
        relabeled.dynamic_generation()
    );
    assert_eq!(baseline.dynamic_generation(), reticked.dynamic_generation());
}

#[test]
fn range_changes_update_static_and_dynamic_generation() {
    let baseline = generation(
        "Direction",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", false),
    );
    let reranged = generation_with_ranges(
        "Direction",
        vec![-180.0, 0.0, 180.0],
        SampleGridPoint::new(0.0, 300.0, "Walk", false),
        [-360.0, 360.0, -100.0, 700.0],
    );

    assert_ne!(baseline.static_generation(), reranged.static_generation());
    assert_ne!(baseline.dynamic_generation(), reranged.dynamic_generation());
}
