use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::hint::black_box;
use std::time::Instant;

use crate::core::extension::{FieldEditorContainer, FieldEditorInstance, FieldEditorKind};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue, ReflectedValue,
};

use super::{
    inspector_plugin_component_project_reflected_properties,
    inspector_plugin_component_property_from_reflected_field,
    InspectorPluginComponentPropertySnapshot,
};

const COMPONENT: &str = "join.Component.Weather";

fn field(key: &str, name: &str, value_type: &str) -> ReflectFieldInfo {
    ReflectFieldInfo::new(
        ReflectFieldId::from_stable_keys(COMPONENT, key),
        name,
        value_type,
        ReflectEditorHint::None,
    )
}

fn value(field: &ReflectFieldInfo, value: ReflectedValue) -> ReflectFieldValue {
    // The schema, rather than the returned diagnostic name, owns presentation.
    ReflectFieldValue::new(field.id, "returned_name_is_not_schema_name", value)
}

fn expected(
    name: &str,
    label: &str,
    value: &str,
    value_kind: &str,
    editable: bool,
    editor: FieldEditorKind,
) -> InspectorPluginComponentPropertySnapshot {
    InspectorPluginComponentPropertySnapshot {
        field_id: format!("{COMPONENT}.{name}"),
        name: name.to_string(),
        label: label.to_string(),
        value: value.to_string(),
        value_kind: value_kind.to_string(),
        editable,
        field_editor: FieldEditorInstance::new(editor),
    }
}

#[test]
fn reflected_projection_joins_reordered_values_and_skips_missing_hidden_fields() {
    let mut zeta = field("zeta", "zeta", "f32");
    zeta.display_name = "weather_amount".to_string();
    let mut alpha = field("alpha", "alpha", "String");
    alpha.display_name = "caption_text".to_string();
    alpha.editable = false;
    let mut hidden = field("hidden", "hidden", "f32");
    hidden.editor_visible = false;
    let missing = field("missing", "missing", "f32");
    let extra = field("extra", "not_in_schema", "f32");
    let returned = vec![
        value(&alpha, ReflectedValue::String("stored".to_string())),
        value(&extra, ReflectedValue::Scalar(999.0)),
        value(&hidden, ReflectedValue::Scalar(77.0)),
        value(&zeta, ReflectedValue::Scalar(0.25)),
    ];
    let schema = vec![zeta, hidden, missing, alpha];
    let drafts = BTreeMap::from([
        (format!("{COMPONENT}.alpha"), "pending caption".to_string()),
        (format!("{COMPONENT}.zeta"), "6.125".to_string()),
    ]);
    let rows = inspector_plugin_component_project_reflected_properties(
        COMPONENT,
        &schema,
        &returned,
        &drafts,
        &FieldEditorContainer::builtin(),
    );
    assert_eq!(
        rows,
        vec![
            expected(
                "alpha",
                "Caption Text",
                "pending caption",
                "String",
                false,
                FieldEditorKind::Auto,
            ),
            expected(
                "zeta",
                "Weather Amount",
                "6.125",
                "f32",
                true,
                FieldEditorKind::Numeric,
            ),
        ]
    );
}

#[test]
fn reflected_projection_uses_first_duplicate_and_stable_name_ties() {
    let mut second = field("second_id", "tie", "String");
    second.display_name = "second_schema".to_string();
    second.editable = false;
    let mut first = field("first_id", "tie", "f32");
    first.display_name = "first_schema".to_string();
    let alpha = field("alpha", "alpha", "f32");
    let returned = vec![
        value(&first, ReflectedValue::Scalar(9.5)),
        value(&alpha, ReflectedValue::Scalar(1.0)),
        value(&first, ReflectedValue::Scalar(999.0)),
        value(&second, ReflectedValue::String("second".to_string())),
    ];
    let rows = inspector_plugin_component_project_reflected_properties(
        COMPONENT,
        &[second, first, alpha],
        &returned,
        &BTreeMap::new(),
        &FieldEditorContainer::builtin(),
    );
    // Equal names retain the complete, distinct schema-order payloads.
    assert_eq!(
        rows,
        vec![
            expected("alpha", "Alpha", "1", "f32", true, FieldEditorKind::Numeric,),
            expected(
                "tie",
                "Second Schema",
                "second",
                "String",
                false,
                FieldEditorKind::Auto,
            ),
            expected(
                "tie",
                "First Schema",
                "9.5",
                "f32",
                true,
                FieldEditorKind::Numeric,
            ),
        ]
    );
}

#[test]
fn reflected_projection_preserves_drafts_read_only_and_value_formatting() {
    let precise = field("precise", "precise", "f32");
    let wind = field("wind", "wind", "Vec3");
    let mut label = field("label", "label", "String");
    label.editable = false;
    let layers = field("layers", "layers", "List");
    let empty = field("empty", "empty", "Null");
    let returned = vec![
        value(&wind, ReflectedValue::Vec3([1.0, 2.0, 3.0])),
        value(&empty, ReflectedValue::Null),
        value(&label, ReflectedValue::String("stored".to_string())),
        value(&precise, ReflectedValue::Scalar(0.125)),
        value(
            &layers,
            ReflectedValue::List(vec![ReflectedValue::Bool(true)]),
        ),
    ];
    let drafts = BTreeMap::from([
        (format!("{COMPONENT}.precise"), "0.123456789".to_string()),
        (format!("{COMPONENT}.label"), "read only draft".to_string()),
        (format!("{COMPONENT}.layers"), "pending list".to_string()),
    ]);
    let rows = inspector_plugin_component_project_reflected_properties(
        COMPONENT,
        &[precise, wind, label, layers, empty],
        &returned,
        &drafts,
        &FieldEditorContainer::builtin(),
    );
    assert_eq!(
        rows,
        vec![
            expected("empty", "Empty", "", "Null", false, FieldEditorKind::Auto,),
            expected(
                "label",
                "Label",
                "read only draft",
                "String",
                false,
                FieldEditorKind::Auto,
            ),
            expected(
                "layers",
                "Layers",
                "pending list",
                "List",
                false,
                FieldEditorKind::Auto,
            ),
            expected(
                "precise",
                "Precise",
                "0.123456789",
                "f32",
                true,
                FieldEditorKind::Numeric,
            ),
            expected(
                "wind",
                "Wind",
                "1, 2, 3",
                "Vec3",
                true,
                FieldEditorKind::Auto,
            ),
        ]
    );
}

// This baseline is retained only for this benchmark. Production always calls
// the shared indexed projection; ordinary regressions assert explicit payloads.
fn legacy_projection(
    component_id: &str,
    schema: &[ReflectFieldInfo],
    returned: &[ReflectFieldValue],
    drafts: &BTreeMap<String, String>,
    editors: &FieldEditorContainer,
) -> Vec<InspectorPluginComponentPropertySnapshot> {
    let mut rows = schema
        .iter()
        .filter(|field| field.editor_visible)
        .filter_map(|field| {
            let value = returned
                .iter()
                .find(|candidate| candidate.field_id == field.id)?;
            Some(inspector_plugin_component_property_from_reflected_field(
                component_id,
                value,
                &field.name,
                &field.display_name,
                &field.value_type_path,
                field.editable,
                drafts,
                editors,
            ))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.name.cmp(&right.name));
    rows
}

struct ProjectionInput {
    schema: Vec<ReflectFieldInfo>,
    returned: Vec<ReflectFieldValue>,
    drafts: BTreeMap<String, String>,
}

const SEED: u64 = 0x5a17_2026_1016_0005;
const WARMUP_PAIRS: usize = 3;
const MEASURED_PAIRS: usize = 31;

fn benchmark_input(count: usize) -> ProjectionInput {
    let mut schema = Vec::with_capacity(count);
    let mut returned = Vec::with_capacity(count);
    let mut drafts = BTreeMap::new();
    for index in 0..count {
        let name = format!("field_{index:05}");
        let mut descriptor = field(&name, &name, "f32");
        descriptor.display_name = format!("Field {index:05}");
        descriptor.editable = index % 7 != 0;
        returned.push(value(
            &descriptor,
            ReflectedValue::Scalar(index as f32 * 0.125),
        ));
        if index % 13 == 0 {
            drafts.insert(format!("{COMPONENT}.{name}"), format!("{index}.0625"));
        }
        schema.push(descriptor);
    }
    // Deterministic Fisher-Yates order, prepared outside every timed interval.
    let mut state = SEED ^ count as u64;
    for index in (1..returned.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let other = (state % (index as u64 + 1)) as usize;
        returned.swap(index, other);
    }
    ProjectionInput {
        schema,
        returned,
        drafts,
    }
}

type Projection = fn(
    &str,
    &[ReflectFieldInfo],
    &[ReflectFieldValue],
    &BTreeMap<String, String>,
    &FieldEditorContainer,
) -> Vec<InspectorPluginComponentPropertySnapshot>;

fn timed_projection(
    projection: Projection,
    input: &ProjectionInput,
    editors: &FieldEditorContainer,
) -> (u64, Vec<InspectorPluginComponentPropertySnapshot>) {
    let started = Instant::now();
    let output = black_box(black_box(projection)(
        black_box(COMPONENT),
        black_box(&input.schema),
        black_box(&input.returned),
        black_box(&input.drafts),
        black_box(editors),
    ));
    let elapsed = u64::try_from(started.elapsed().as_nanos()).expect("elapsed nanoseconds fit u64");
    // Output remains alive until both calls finish. Equality, checksum and both
    // output drops happen after timing; alternating AB/BA balances that lifetime.
    (elapsed, output)
}

fn timed_pair(
    input: &ProjectionInput,
    editors: &FieldEditorContainer,
    baseline_first: bool,
) -> (
    (u64, Vec<InspectorPluginComponentPropertySnapshot>),
    (u64, Vec<InspectorPluginComponentPropertySnapshot>),
) {
    if baseline_first {
        let baseline = timed_projection(legacy_projection, input, editors);
        let optimized = timed_projection(
            inspector_plugin_component_project_reflected_properties,
            input,
            editors,
        );
        (baseline, optimized)
    } else {
        let optimized = timed_projection(
            inspector_plugin_component_project_reflected_properties,
            input,
            editors,
        );
        let baseline = timed_projection(legacy_projection, input, editors);
        (baseline, optimized)
    }
}

fn checksum(rows: &[InspectorPluginComponentPropertySnapshot]) -> u64 {
    fn feed(hash: &mut u64, bytes: &[u8]) {
        for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
            *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
    }
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for row in rows {
        for text in [
            row.field_id.as_str(),
            row.name.as_str(),
            row.label.as_str(),
            row.value.as_str(),
            row.value_kind.as_str(),
            row.field_editor.kind().as_str(),
        ] {
            feed(&mut hash, text.as_bytes());
        }
        feed(&mut hash, &[u8::from(row.editable)]);
        for marker in row.field_editor.asset_reference_markers() {
            feed(&mut hash, marker.as_bytes());
        }
    }
    hash
}

fn p95(samples: &[u64]) -> u64 {
    assert_eq!(samples.len(), MEASURED_PAIRS);
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[29] // nearest-rank ceil(0.95 * 31) = 30, zero-based index 29.
}

#[test]
#[ignore = "Release-only projection benchmark; requires owner source/config seals and raw paired evidence"]
fn reflected_projection_linear_join_p95_1_100_10000() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored producer with --release"
    );
    let source_identity = std::env::var("ZIRCON_EDITOR05_SOURCE_IDENTITY")
        .expect("owner actual preseal source canonical is required");
    assert!(
        source_identity.len() == 64 && source_identity.bytes().all(|byte| byte.is_ascii_hexdigit())
    );
    let config_identity = std::env::var("ZIRCON_EDITOR05_CONFIG_IDENTITY")
        .expect("owner actual compiler/config seal identity is required");
    assert!(!config_identity.trim().is_empty());
    let editors = FieldEditorContainer::builtin();
    for count in [1, 100, 10_000] {
        // Typed projection DTOs are not proof of Scene admission at 10k fields.
        let input = benchmark_input(count);
        for pair in 0..WARMUP_PAIRS {
            let ((_, baseline), (_, optimized)) = timed_pair(&input, &editors, pair % 2 == 0);
            assert_eq!(baseline, optimized);
            drop((baseline, optimized));
        }
        let mut baseline_samples = Vec::with_capacity(MEASURED_PAIRS);
        let mut optimized_samples = Vec::with_capacity(MEASURED_PAIRS);
        for pair in 0..MEASURED_PAIRS {
            let baseline_first = pair % 2 == 0;
            let ((baseline_ns, baseline), (optimized_ns, optimized)) =
                timed_pair(&input, &editors, baseline_first);
            assert_eq!(baseline, optimized, "scale={count} pair={pair}");
            assert_eq!(optimized.len(), count);
            let output_checksum = checksum(&optimized);
            println!(
                "{}",
                serde_json::json!({
                    "producer": "editor05-reflected-projection-v1",
                    "measurement": "property_projection_only",
                    "source_identity": &source_identity,
                    "config_identity": &config_identity,
                    "package": env!("CARGO_PKG_NAME"),
                    "package_version": env!("CARGO_PKG_VERSION"),
                    "module": module_path!(),
                    "os": std::env::consts::OS,
                    "arch": std::env::consts::ARCH,
                    "pointer_bits": usize::BITS,
                    "debug_assertions": cfg!(debug_assertions),
                    "scale": count,
                    "seed": SEED,
                    "shuffle_seed": SEED ^ count as u64,
                    "pair": pair,
                    "order": if baseline_first { "AB" } else { "BA" },
                    "warmup_pairs_excluded": WARMUP_PAIRS,
                    "schema_field_count": input.schema.len(),
                    "returned_field_count": input.returned.len(),
                    "row_count": optimized.len(),
                    "baseline_ns": baseline_ns,
                    "optimized_ns": optimized_ns,
                    "semantic_checksum_fnv1a64": format!("{output_checksum:016x}"),
                    "semantic_equal": true,
                    "timer_boundary": "call_inputs_output_black_box; equality_checksum_drops_outside; both_outputs_live_until_pair_complete"
                })
            );
            baseline_samples.push(baseline_ns);
            optimized_samples.push(optimized_ns);
            drop((baseline, optimized));
        }
        let baseline_p95_ns = p95(&baseline_samples);
        let optimized_p95_ns = p95(&optimized_samples);
        println!(
            "{}",
            serde_json::json!({
                "producer": "editor05-reflected-projection-v1",
                "measurement": "property_projection_only",
                "source_identity": &source_identity,
                "config_identity": &config_identity,
                "scale": count,
                "measured_pairs": MEASURED_PAIRS,
                "ab_pairs": 16,
                "ba_pairs": 15,
                "quantile": "per_algorithm_per_scale_nearest_rank",
                "p95_zero_based_index": 29,
                "baseline_p95_ns": baseline_p95_ns,
                "optimized_p95_ns": optimized_p95_ns,
                "primary_gate_applies": count == 10_000,
                "absolute_budget_ns": null,
                "rss_budget_bytes": null,
                "excluded": ["scene_read", "schema_admission", "world_lock", "full_snapshot", "GUI"],
                "drop_timing": "outside_each_call_and_after_both_pair_outputs"
            })
        );
        if count == 10_000 {
            assert!(
                u128::from(optimized_p95_ns) * 100 <= u128::from(baseline_p95_ns) * 50,
                "10k projection gate: optimized_p95={optimized_p95_ns} baseline_p95={baseline_p95_ns}"
            );
        }
    }
}
