use std::hint::black_box;

use super::*;

#[test]
fn export_wizard_progress_parses_cli_stream_into_stage_snapshots() {
    let mut progress = ExportWizardProgressState::new();

    let event = progress
        .push_stdout_line("zircon_export stage=CookAssets profile=windows-release")
        .expect("stage banner should produce a progress event");
    assert_eq!(event.stage, ExportStage::CookAssets);
    assert_eq!(event.kind, ExportStageProgressKind::Running);

    progress.push_stdout_line("cooked_asset_manifest=D:\\export\\stages\\cook_assets\\assets.json");
    progress.push_stdout_line("report=D:\\export\\stages\\cook_assets\\report.json");
    progress.push_stdout_line(r#""fatal": false,"#);

    let cook_assets = progress
        .snapshot(ExportStage::CookAssets)
        .expect("CookAssets snapshot should exist");
    assert_eq!(cook_assets.kind, ExportStageProgressKind::Passed);
    assert_eq!(cook_assets.profile.as_deref(), Some("windows-release"));
    assert_eq!(
        cook_assets.report_path.as_deref(),
        Some("D:\\export\\stages\\cook_assets\\report.json")
    );
    assert!(cook_assets.artifact_paths.iter().any(|artifact| {
        artifact.key == "cooked_asset_manifest"
            && artifact.path == "D:\\export\\stages\\cook_assets\\assets.json"
    }));
}

#[test]
fn export_wizard_progress_marks_fatal_stage_reports() {
    let mut progress = ExportWizardProgressState::new();

    progress.push_stdout_line("zircon_export stage=PlatformBundle profile=windows-release");
    progress.push_stdout_line("bundle=D:\\export\\bundle\\windows-release");
    progress.push_stdout_line(r#""fatal": true,"#);

    let platform_bundle = progress
        .snapshot(ExportStage::PlatformBundle)
        .expect("PlatformBundle snapshot should exist");
    assert_eq!(platform_bundle.kind, ExportStageProgressKind::Fatal);
    assert!(platform_bundle
        .artifact_paths
        .iter()
        .any(|artifact| artifact.key == "bundle"));
}

#[test]
fn export_wizard_progress_ignores_pipeline_summary_json_lines() {
    let mut progress = ExportWizardProgressState::new();

    progress.push_stdout_line("zircon_export stage=Report profile=windows-release");
    progress.push_stdout_line(r#"  "export_plan": {"#);
    progress.push_stdout_line(r#"    "unsupported_strategies": ["#);
    progress.push_stdout_line(r#"      "future_error_path""#);
    progress.push_stdout_line(r#"    ]"#);
    progress.push_stdout_line(r#"  },"#);
    progress.push_stdout_line(r#""fatal": false,"#);

    let report = progress
        .snapshot(ExportStage::Report)
        .expect("Report snapshot should exist");
    assert_eq!(report.kind, ExportStageProgressKind::Passed);
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
}

#[test]
fn export_wizard_progress_records_report_diagnostics_array_lines() {
    let mut progress = ExportWizardProgressState::new();

    progress.push_stdout_line("zircon_export stage=Report profile=windows-release");
    progress.push_stdout_line(r#""diagnostics": ["#);
    progress.push_stdout_line(r#"  "validate failed","#);
    progress.push_stdout_line(r#"],"#);
    progress.push_stdout_line(r#""fatal": true,"#);

    let report = progress
        .snapshot(ExportStage::Report)
        .expect("Report snapshot should exist");
    assert_eq!(report.kind, ExportStageProgressKind::Fatal);
    assert_eq!(report.diagnostics, vec!["validate failed"]);
}

#[test]
fn export_pipeline_stage_parser_accepts_cli_and_report_stage_names() {
    assert_eq!(
        "source_template".parse::<ExportStage>().ok(),
        Some(ExportStage::SourceTemplate)
    );
    assert_eq!(
        "SourceTemplate".parse::<ExportStage>().ok(),
        Some(ExportStage::SourceTemplate)
    );
    assert_eq!(
        "platform_bundle".parse::<ExportStage>().ok(),
        Some(ExportStage::PlatformBundle)
    );
    assert_eq!(
        "native_dynamic".parse::<ExportStage>().ok(),
        Some(ExportStage::NativeDynamic)
    );
    assert_eq!(
        "NativeDynamic".parse::<ExportStage>().ok(),
        Some(ExportStage::NativeDynamic)
    );
    assert_eq!(
        ExportStage::ALL,
        [
            ExportStage::Validate,
            ExportStage::SourceTemplate,
            ExportStage::NativeDynamic,
            ExportStage::CompileHost,
            ExportStage::CookAssets,
            ExportStage::Pack,
            ExportStage::PlatformBundle,
            ExportStage::Report,
        ]
    );
}

#[test]
fn optimization_batch_fr_editor404_reserves_exact_export_stage_capacity() {
    let stages = export_pipeline_stages_for_strategies(&[
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::LibraryEmbed,
    ]);

    assert_eq!(stages, ExportStage::ALL);
    assert_eq!(stages.capacity(), stages.len());
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fr_editor404_exact_export_stage_capacity_benchmark() {
    const SAMPLE_PAIRS: usize = 17;
    const BUILDS_PER_SAMPLE: usize = 131_072;
    let strategies = [
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::LibraryEmbed,
    ];

    for _ in 0..4 {
        black_box(measure_stage_builds(&strategies, false, BUILDS_PER_SAMPLE));
        black_box(measure_stage_builds(&strategies, true, BUILDS_PER_SAMPLE));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_stage_builds(&strategies, false, BUILDS_PER_SAMPLE));
            optimized_samples.push(measure_stage_builds(&strategies, true, BUILDS_PER_SAMPLE));
        } else {
            optimized_samples.push(measure_stage_builds(&strategies, true, BUILDS_PER_SAMPLE));
            legacy_samples.push(measure_stage_builds(&strategies, false, BUILDS_PER_SAMPLE));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR404_EXACT_EXPORT_STAGE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} builds_per_sample={BUILDS_PER_SAMPLE} stages_per_build={} legacy_growth_allocations_per_build=1 optimized_growth_allocations_per_build=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        ExportStage::ALL.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 75 / 100);
}

fn measure_stage_builds(
    strategies: &[ExportPackagingStrategy],
    optimized: bool,
    builds: usize,
) -> u128 {
    let started = std::time::Instant::now();
    for _ in 0..builds {
        let stages = if optimized {
            export_pipeline_stages_for_strategies(black_box(strategies))
        } else {
            legacy_export_pipeline_stages_for_strategies(black_box(strategies))
        };
        black_box(stages);
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_export_pipeline_stages_for_strategies(
    strategies: &[ExportPackagingStrategy],
) -> Vec<ExportStage> {
    let mut stages = Vec::new();
    stages.push(ExportStage::Validate);
    if strategies.contains(&ExportPackagingStrategy::SourceTemplate) {
        push_stage_once(&mut stages, ExportStage::SourceTemplate);
    }
    if strategies.contains(&ExportPackagingStrategy::NativeDynamic) {
        push_stage_once(&mut stages, ExportStage::NativeDynamic);
        push_stage_once(&mut stages, ExportStage::CompileHost);
        push_stage_once(&mut stages, ExportStage::CookAssets);
        push_stage_once(&mut stages, ExportStage::Pack);
        push_stage_once(&mut stages, ExportStage::PlatformBundle);
    }
    if strategies.contains(&ExportPackagingStrategy::LibraryEmbed) {
        push_stage_once(&mut stages, ExportStage::CompileHost);
        push_stage_once(&mut stages, ExportStage::CookAssets);
        push_stage_once(&mut stages, ExportStage::Pack);
        push_stage_once(&mut stages, ExportStage::PlatformBundle);
    }
    stages.push(ExportStage::Report);
    stages
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
