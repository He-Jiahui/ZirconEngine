use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_ht_editor602_borrowed_projection_preserves_view_command() {
    let mut batch = ContributionBatch::default()
        .with_required_capabilities(["editor.sample.enabled", "editor.sample.audit"]);
    let view = ViewDescriptor::new("plugin.sample.dashboard", "Dashboard", "Plugins");
    let operation = view.open_operation_path().unwrap();
    batch.register_view(view).unwrap();

    let mut registry = EditorCommandRegistry::default_workbench();
    project_extension_commands(&mut registry, &batch).unwrap();

    let command = registry
        .command(operation.as_str())
        .expect("projected view command");
    assert_eq!(
        command.required_capabilities(),
        &[
            "editor.sample.audit".to_string(),
            "editor.sample.enabled".to_string()
        ]
    );
    assert_eq!(
        command.event(),
        Some(&EditorEvent::WorkbenchMenu(MenuAction::OpenView(
            ViewDescriptorId::new("plugin.sample.dashboard"),
        )))
    );
}

#[test]
fn optimization_batch_ht_editor602_projection_does_not_clone_entire_batch() {
    let source = include_str!("../../contribution.rs");
    let projection = source
        .split("fn project_extension_commands")
        .nth(1)
        .expect("extension projection")
        .split("fn asset_write_targets")
        .next()
        .expect("bounded extension projection");

    assert!(!projection.contains("source_extension.clone()"));
    assert!(!projection.contains("views().into_iter().cloned().collect"));
    assert!(projection.contains(".pending_commands()"));
    assert!(projection.contains(".collect::<Vec<_>>()"));
    assert!(projection.contains("source_extension.operation_factory(command.id()).cloned()"));
}

fn benchmark_batch(view_count: usize) -> ContributionBatch {
    let mut batch = ContributionBatch::default().with_required_capabilities(
        (0..32).map(|index| format!("editor.performance.capability.{index:03}")),
    );
    for index in 0..view_count {
        batch
            .register_view(ViewDescriptor::new(
                format!("plugin.performance.view_{index:05}"),
                format!(
                    "Performance View {index:05} {}",
                    "retained-view-payload".repeat(4)
                ),
                "Performance",
            ))
            .unwrap();
    }
    for index in 0..8 {
        batch
            .register_command(EditorCommandDescriptor::operation(
                EditorOperationPath::parse(format!("plugin.performance.command_{index:03}"))
                    .unwrap(),
            ))
            .unwrap();
    }
    batch
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ht_editor602_selective_projection_performance_evidence() {
    let batch = benchmark_batch(4_096);
    const SAMPLE_PAIRS: usize = 17;
    let measure_legacy = || {
        let started = Instant::now();
        black_box(ContributionBatch::clone(black_box(&batch)));
        started.elapsed().as_nanos().max(1)
    };
    let measure_selective = || {
        let started = Instant::now();
        let commands = black_box(&batch)
            .pending_commands()
            .cloned()
            .collect::<Vec<_>>();
        black_box((
            commands,
            batch.views().count(),
            batch.required_capabilities().len(),
        ));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_selective());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut selective_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            selective_samples.push(measure_selective());
        } else {
            selective_samples.push(measure_selective());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    selective_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let selective_p50 = selective_samples[8];
    let selective_p95 = selective_samples[16];
    println!(
        "EDITOR602_SELECTIVE_COMMAND_PROJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 selective_first_pairs=8 views={} commands=8 capabilities=32 legacy_p50_ns={} legacy_p95_ns={} selective_p50_ns={} selective_p95_ns={} legacy_view_descriptor_clones={} selective_view_descriptor_clones=0 target_ratio_bp=1000",
        batch.views().count(),
        legacy_p50,
        legacy_p95,
        selective_p50,
        selective_p95,
        batch.views().count(),
    );
    assert!(
        selective_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(1_000),
        "selective projection P95 {selective_p95} ns exceeded 10% of legacy {legacy_p95} ns"
    );
}
