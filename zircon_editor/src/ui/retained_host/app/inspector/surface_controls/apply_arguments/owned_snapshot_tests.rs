use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::{
    core::extension::FieldEditorInstance,
    ui::workbench::snapshot::{
        InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot,
    },
};

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828id_editor_inspector_apply_moves_snapshot_strings() {
    let inspector = benchmark_inspector(4 * 1024);
    let expected = [
        inspector.name.as_ptr(),
        inspector.parent.as_ptr(),
        inspector.translation[0].as_ptr(),
        inspector.plugin_components[0].properties[0]
            .field_id
            .as_ptr(),
        inspector.plugin_components[0].properties[0].value.as_ptr(),
    ];

    let arguments = inspector_apply_arguments_from_snapshot(inspector);
    let changes = binding_array(&arguments[1]);
    let actual = [
        change_value(changes, 0).as_ptr(),
        change_value(changes, 1).as_ptr(),
        change_value(changes, 2).as_ptr(),
        change_key(changes, 5).as_ptr(),
        change_value(changes, 5).as_ptr(),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn optimization_batch_20260828id_editor_inspector_apply_consumes_owned_snapshot_fields() {
    let source = include_str!("../apply_arguments.rs");
    let entry = source
        .split("pub(super) fn inspector_apply_arguments")
        .nth(1)
        .and_then(|body| {
            body.split("fn inspector_apply_arguments_from_snapshot")
                .next()
        })
        .expect("inspector apply entry implementation");
    let conversion = source
        .split("fn inspector_apply_arguments_from_snapshot")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("owned inspector snapshot conversion");

    assert!(entry.contains("inspector_apply_arguments_from_snapshot(inspector)"));
    assert!(conversion.contains("let [translation_x, translation_y, translation_z]"));
    assert!(conversion.contains("plugin_components.into_iter()"));
    assert!(conversion.contains("component.properties.into_iter()"));
    assert!(!conversion.contains("inspector.name.clone()"));
    assert!(!conversion.contains("property.value.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828id_editor_owned_inspector_apply_arguments_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 128;

    black_box(legacy_inspector_apply_arguments(benchmark_inspector(
        32 * 1024,
    )));
    black_box(inspector_apply_arguments_from_snapshot(
        benchmark_inspector(32 * 1024),
    ));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| benchmark_inspector(32 * 1024))
            .collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS)
            .map(|_| benchmark_inspector(32 * 1024))
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_inspectors(
                legacy_inputs,
                legacy_inspector_apply_arguments,
            ));
            optimized_samples.push(measure_inspectors(
                optimized_inputs,
                inspector_apply_arguments_from_snapshot,
            ));
        } else {
            optimized_samples.push(measure_inspectors(
                optimized_inputs,
                inspector_apply_arguments_from_snapshot,
            ));
            legacy_samples.push(measure_inspectors(
                legacy_inputs,
                legacy_inspector_apply_arguments,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR222_OWNED_INSPECTOR_APPLY_ARGUMENTS_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_inspector(field_bytes: usize) -> InspectorSnapshot {
    let large = |prefix: &str| format!("{prefix}{}", "x".repeat(field_bytes));
    InspectorSnapshot {
        id: 2,
        name: large("name/"),
        parent: large("parent/"),
        translation: [
            large("translation-x/"),
            large("translation-y/"),
            large("translation-z/"),
        ],
        scale: ["1".to_owned(), "1".to_owned(), "1".to_owned()],
        render_layer_mask: 1,
        plugin_components: vec![InspectorPluginComponentSnapshot {
            component_id: "component".to_owned(),
            display_name: "Component".to_owned(),
            plugin_id: "plugin".to_owned(),
            customization_available: true,
            customization_ui_document: None,
            customization_controller: None,
            customization_template_id: None,
            customization_data_root: None,
            customization_bindings: Vec::new(),
            diagnostic: None,
            properties: vec![InspectorPluginComponentPropertySnapshot {
                field_id: large("field/"),
                name: "field".to_owned(),
                label: "Field".to_owned(),
                value: large("value/"),
                value_kind: "string".to_owned(),
                editable: true,
                field_editor: FieldEditorInstance::automatic(),
            }],
        }],
    }
}

fn legacy_inspector_apply_arguments(inspector: InspectorSnapshot) -> Vec<UiBindingValue> {
    let parent_value = if inspector.parent.trim().is_empty() {
        UiBindingValue::Null
    } else {
        UiBindingValue::string(inspector.parent.clone())
    };
    let mut changes = vec![
        change("name", inspector.name.clone()),
        UiBindingValue::array(vec![UiBindingValue::string("parent"), parent_value]),
        change("transform.translation.x", inspector.translation[0].clone()),
        change("transform.translation.y", inspector.translation[1].clone()),
        change("transform.translation.z", inspector.translation[2].clone()),
    ];
    changes.extend(
        inspector
            .plugin_components
            .iter()
            .filter(|component| component.customization_available)
            .flat_map(|component| component.properties.iter())
            .filter(|property| property.editable)
            .map(|property| change(property.field_id.clone(), property.value.clone())),
    );
    vec![
        UiBindingValue::string("entity://selected"),
        UiBindingValue::array(changes),
    ]
}

fn change(key: impl Into<String>, value: impl Into<String>) -> UiBindingValue {
    UiBindingValue::array(vec![
        UiBindingValue::string(key),
        UiBindingValue::string(value),
    ])
}

fn binding_array(value: &UiBindingValue) -> &[UiBindingValue] {
    let UiBindingValue::Array(values) = value else {
        panic!("expected binding array");
    };
    values
}

fn change_key(changes: &[UiBindingValue], index: usize) -> &str {
    binding_array(&changes[index])[0]
        .as_str()
        .expect("change key")
}

fn change_value(changes: &[UiBindingValue], index: usize) -> &str {
    binding_array(&changes[index])[1]
        .as_str()
        .expect("change value")
}

fn measure_inspectors(
    inspectors: Vec<InspectorSnapshot>,
    mut convert: impl FnMut(InspectorSnapshot) -> Vec<UiBindingValue>,
) -> u128 {
    let started = Instant::now();
    for inspector in inspectors {
        black_box(convert(black_box(inspector)));
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
