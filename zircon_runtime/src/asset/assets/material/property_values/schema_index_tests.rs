use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use crate::asset::{AssetUri, ShaderSourceLanguage};
use crate::core::framework::render::ShaderAssetKind;

const SCHEMA_SIZE: usize = 4_096;
const OVERRIDE_SIZE: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn runtime09c_batch_material_property_schema_rescan_preserves_projection() {
    let material = material_with_overrides(&[
        ("declared_float", toml::Value::Float(2.5)),
        ("declared_invalid", toml::Value::Boolean(true)),
        ("declared_string", toml::Value::String("kept".to_string())),
        (
            "unknown_string",
            toml::Value::String("fallback".to_string()),
        ),
        ("unknown_number", toml::Value::Integer(7)),
    ]);
    let shader = shader_with_schema(vec![
        property("declared_float", MaterialPropertyKind::Float),
        property("declared_invalid", MaterialPropertyKind::Float),
        property("declared_string", MaterialPropertyKind::Float),
    ]);

    assert_eq!(
        shader_property_values_for_shader(&material, &shader),
        legacy_shader_property_values(&material, &shader).0
    );
}

#[test]
fn runtime09c_batch_material_property_schema_rescan_eliminates_pairwise_work() {
    let material = scale_material();
    let shader = scale_shader();
    let (_, schema_comparisons) = legacy_shader_property_values(&material, &shader);
    let projected = shader_property_values_for_shader(&material, &shader);

    assert_eq!(schema_comparisons, SCHEMA_SIZE * OVERRIDE_SIZE);
    assert_eq!(projected.len(), OVERRIDE_SIZE);
    let source = include_str!("../property_values.rs");
    let fallback_loop = source
        .split("for (name, value) in material.shader_property_overrides()")
        .nth(1)
        .expect("property fallback loop must remain")
        .split("fn render_property_value")
        .next()
        .expect("property fallback loop must terminate");
    assert!(!fallback_loop.contains("property_schema"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn runtime09c_batch_material_property_schema_rescan_p95() {
    let material = scale_material();
    let shader = scale_shader();
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| {
                legacy_shader_property_values(&material, &shader).0
            }));
            optimized.push(measure(|| {
                shader_property_values_for_shader(&material, &shader)
            }));
        } else {
            optimized.push(measure(|| {
                shader_property_values_for_shader(&material, &shader)
            }));
            baseline.push(measure(|| {
                legacy_shader_property_values(&material, &shader).0
            }));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "RUNTIME09C_MATERIAL_PROPERTY_SCHEMA_RESCAN_BENCH_V1 schema_properties={SCHEMA_SIZE} overrides={OVERRIDE_SIZE} sample_pairs={SAMPLE_COUNT} pair_order=alternating_legacy_even baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} schema_comparisons_before={} schema_comparisons_after=0",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        SCHEMA_SIZE * OVERRIDE_SIZE,
    );
    assert!(
        reduction >= 80.0,
        "expected at least 80% P95 reduction, got {reduction:.2}%"
    );
}

fn legacy_shader_property_values(
    material: &MaterialAsset,
    shader: &ShaderAsset,
) -> (BTreeMap<String, RenderMaterialPropertyValue>, usize) {
    let mut values = BTreeMap::new();
    for property in &shader.property_schema {
        let value = material
            .shader_property_override(&property.name)
            .or(property.default.as_ref());
        let Some(value) = value else {
            continue;
        };
        if let Some(value) = render_property_value(property, value) {
            values.insert(property.name.clone(), value);
        }
    }
    let mut schema_comparisons = 0;
    for (name, value) in material.shader_property_overrides() {
        if values.contains_key(name)
            || shader.property_schema.iter().any(|property| {
                schema_comparisons += 1;
                property.name == *name
            })
        {
            continue;
        }
        if let Some(value) = string_property_value(value) {
            values.insert(name.clone(), value);
        }
    }
    (values, schema_comparisons)
}

fn material_with_overrides(entries: &[(&str, toml::Value)]) -> MaterialAsset {
    let mut material = empty_material();
    material.property_values = entries
        .iter()
        .map(|(name, value)| (name.to_string(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    material
}

fn scale_material() -> MaterialAsset {
    let mut material = empty_material();
    material.property_values = (0..OVERRIDE_SIZE)
        .map(|index| {
            (
                format!("unknown_{index:05}"),
                toml::Value::String(format!("value-{index}")),
            )
        })
        .collect::<BTreeMap<_, _>>();
    material
}

fn empty_material() -> MaterialAsset {
    MaterialAsset::from_toml_str(
        "version = 2\n[shader]\nuuid = \"00000000-0000-0000-0000-000000000001\"\nurl = \"res://shaders/scale.zshader\"\n",
    )
    .unwrap()
}

fn scale_shader() -> ShaderAsset {
    shader_with_schema(
        (0..SCHEMA_SIZE)
            .map(|index| property(&format!("declared_{index:05}"), MaterialPropertyKind::Float))
            .collect(),
    )
}

fn property(name: &str, kind: MaterialPropertyKind) -> ShaderMaterialPropertyAsset {
    ShaderMaterialPropertyAsset {
        name: name.to_string(),
        kind,
        required: false,
        default: None,
        editor: BTreeMap::new(),
    }
}

fn shader_with_schema(property_schema: Vec<ShaderMaterialPropertyAsset>) -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/scale.zshader").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: String::new(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema,
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn measure<T>(work: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(work());
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn percent_reduction(before: Duration, after: Duration) -> f64 {
    if before.is_zero() {
        return 0.0;
    }
    100.0 * (before.as_secs_f64() - after.as_secs_f64()) / before.as_secs_f64()
}
