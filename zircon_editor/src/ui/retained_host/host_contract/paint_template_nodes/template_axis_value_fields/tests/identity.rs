use super::{is_workbench_axis_value_field, transform_axis_value_id};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

#[test]
fn optimization_batch_gp_editor428_transform_axis_suffix_dispatch_preserves_rules() {
    let node = TemplatePaneNodeData {
        role: "InputField".to_owned(),
        control_id: "WorkbenchTransformPositionX".to_owned(),
        ..TemplatePaneNodeData::default()
    };
    assert!(is_workbench_axis_value_field(&node));
    assert!(transform_axis_value_id("WorkbenchTransformRotationY").is_some());
    assert!(transform_axis_value_id("WorkbenchTransformScaleZ").is_some());
    assert!(transform_axis_value_id("WorkbenchTransformPositionQ").is_none());
    assert!(transform_axis_value_id("WorkbenchTransformPosition").is_none());
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gp_editor428_transform_axis_suffix_dispatch_benchmark() {
    const MARKER: &str = "EDITOR428_TRANSFORM_AXIS_SUFFIX_DISPATCH_BENCH_V1";
    const ITERATIONS: usize = 100_000;
    let control_id = "WorkbenchTransformPositionX";
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        assert!(transform_axis_value_id(control_id).is_some());
    }
    let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        let field = control_id.strip_prefix("WorkbenchTransform");
        let result = field.and_then(|field| {
            let axis = if field.ends_with('X') {
                "X"
            } else if field.ends_with('Y') {
                "Y"
            } else if field.ends_with('Z') {
                "Z"
            } else {
                return None;
            };
            field
                .strip_suffix(axis)
                .filter(|kind| matches!(*kind, "Position" | "Rotation" | "Scale"))
        });
        assert!(result.is_some());
    }
    let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    eprintln!(
        "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
}
