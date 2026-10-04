use super::*;
use std::hint::black_box;
use std::time::{Duration, Instant};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;

#[test]
fn terrain_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("terrain authoring registration");
    let operation =
        EditorOperationPath::parse("terrain.authoring.import_heightfield").expect("operation path");
    let descriptor = registry
        .commands()
        .command(&operation)
        .expect("import heightfield operation registered");

    assert_eq!(
        descriptor
            .menu_path()
            .expect("heightfield command menu path")
            .stable_path(),
        "plugins/terrain/terrain.authoring.import_heightfield"
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("terrain.import_heightfield.v1")
    );
    assert!(registry.menu_items().is_empty());
}

#[test]
fn terrain_heightfield_import_accepts_supported_extensions_and_matching_samples() {
    let diagnostics = validate_heightfield_import(&TerrainHeightfieldImportRequest {
        width: 4,
        height: 4,
        sample_count: Some(16),
        source_extension: ".r16".to_string(),
    });

    assert!(diagnostics.is_empty());
    assert_eq!(
        terrain_import_output_kind("png"),
        Some("terrain.heightfield")
    );
}

#[test]
fn terrain_import_plan_reports_typed_heightfield_format() {
    let request = TerrainHeightfieldImportRequest {
        width: 8,
        height: 4,
        sample_count: Some(32),
        source_extension: ".PNG".to_string(),
    };

    let heightfield = plan_terrain_import(TerrainImportKind::Heightfield, &request)
        .expect("heightfield import request is valid");

    assert_eq!(heightfield.normalized_extension, "png");
    assert_eq!(
        heightfield.source_format,
        TerrainHeightfieldSourceFormat::Png
    );
    assert_eq!(heightfield.output_kind, "terrain.heightfield");
    assert_eq!(heightfield.expected_sample_count, 32);
}

#[test]
fn terrain_import_plan_rejects_layer_stack_through_heightfield_contract() {
    let request = TerrainHeightfieldImportRequest {
        width: 8,
        height: 4,
        sample_count: Some(32),
        source_extension: "raw".to_string(),
    };

    let diagnostics = plan_terrain_import(TerrainImportKind::LayerStack, &request)
        .expect_err("layer stacks require a layer-aware import contract");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("layer stack import requires")));
}

#[test]
fn terrain_heightfield_import_reports_invalid_dimensions_extension_and_sample_count() {
    let mut diagnostics = validate_heightfield_import(&TerrainHeightfieldImportRequest {
        width: 0,
        height: 4,
        sample_count: None,
        source_extension: "exr".to_string(),
    });
    diagnostics.extend(validate_heightfield_import(
        &TerrainHeightfieldImportRequest {
            width: 2,
            height: 4,
            sample_count: Some(5),
            source_extension: "raw".to_string(),
        },
    ));

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("dimensions must be greater")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("extension `exr` is not supported")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("expected 8 samples")));
}

#[test]
#[ignore = "release performance gate"]
fn terrain_import_planning_release_gate_avoids_duplicate_extension_allocations() {
    const SAMPLE_PAIRS: usize = 21;
    const REQUESTS_PER_SAMPLE: usize = 16_384;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

    let request = TerrainHeightfieldImportRequest {
        width: 4_096,
        height: 4_096,
        sample_count: Some(16_777_216),
        source_extension: "  .PNG  ".to_string(),
    };

    for _ in 0..2 {
        black_box(measure_terrain_import_planning(
            &request,
            REQUESTS_PER_SAMPLE,
            legacy_heightfield_import_projection,
        ));
        black_box(measure_terrain_import_planning(
            &request,
            REQUESTS_PER_SAMPLE,
            optimized_heightfield_import_projection,
        ));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_terrain_import_planning(
                &request,
                REQUESTS_PER_SAMPLE,
                legacy_heightfield_import_projection,
            ));
            optimized_samples.push(measure_terrain_import_planning(
                &request,
                REQUESTS_PER_SAMPLE,
                optimized_heightfield_import_projection,
            ));
        } else {
            optimized_samples.push(measure_terrain_import_planning(
                &request,
                REQUESTS_PER_SAMPLE,
                optimized_heightfield_import_projection,
            ));
            legacy_samples.push(measure_terrain_import_planning(
                &request,
                REQUESTS_PER_SAMPLE,
                legacy_heightfield_import_projection,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_p95(&optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT plugins08_terrain_typed_import sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even requests_per_sample={REQUESTS_PER_SAMPLE} legacy_extension_allocations_per_request=2 optimized_extension_allocations_per_request=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}",
        durations_csv(&legacy_samples),
        durations_csv(&optimized_samples),
    );

    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "typed terrain import planning must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn legacy_heightfield_import_projection(
    request: &TerrainHeightfieldImportRequest,
) -> Option<(String, usize)> {
    let validation_extension = request
        .source_extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if request.width == 0
        || request.height == 0
        || !matches!(validation_extension.as_str(), "raw" | "r16" | "png")
        || request.sample_count != Some(request.width as usize * request.height as usize)
    {
        return None;
    }
    let normalized_extension = request
        .source_extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    Some((
        normalized_extension,
        request.width as usize * request.height as usize,
    ))
}

fn optimized_heightfield_import_projection(
    request: &TerrainHeightfieldImportRequest,
) -> Option<(String, usize)> {
    plan_terrain_import(TerrainImportKind::Heightfield, request)
        .ok()
        .map(|plan| (plan.normalized_extension, plan.expected_sample_count))
}

fn measure_terrain_import_planning(
    request: &TerrainHeightfieldImportRequest,
    requests_per_sample: usize,
    planner: fn(&TerrainHeightfieldImportRequest) -> Option<(String, usize)>,
) -> Duration {
    let started = Instant::now();
    for _ in 0..requests_per_sample {
        black_box(planner(black_box(request)));
    }
    started.elapsed()
}

fn nearest_rank_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}
