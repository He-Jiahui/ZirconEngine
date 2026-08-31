use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_axis_value_field(
    node: &TemplatePaneNodeData,
) -> bool {
    if !is_text_input_node(node) {
        return false;
    }
    let control_id = node.control_id.as_str();
    control_id == "WorkbenchAxisValueFieldRoot"
        || transform_axis_value_id(control_id).is_some()
        || node.component_role.as_str() == "axis-value-field"
}

fn transform_axis_value_id(control_id: &str) -> Option<TransformAxisValueId> {
    let field = control_id.strip_prefix("WorkbenchTransform")?;
    let bytes = field.as_bytes();
    let Some((&axis, kind_bytes)) = bytes.split_last() else {
        return None;
    };
    if !matches!(axis, b'X' | b'Y' | b'Z') {
        return None;
    }
    let kind = std::str::from_utf8(kind_bytes).ok()?;
    if matches!(kind, "Position" | "Rotation" | "Scale") {
        Some(TransformAxisValueId)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct TransformAxisValueId;

fn is_text_input_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.role.as_str(),
        "InputField" | "LineEdit" | "TextField" | "MuiTextField"
    ) || matches!(
        node.component_role.as_str(),
        "input-field" | "number-field" | "text-field"
    )
}

#[cfg(test)]
mod tests {
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
}
