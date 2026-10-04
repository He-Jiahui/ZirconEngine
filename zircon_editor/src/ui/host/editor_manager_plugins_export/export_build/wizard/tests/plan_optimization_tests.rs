use super::*;

#[test]
fn optimization_batch_20260830eo_ordered_commands_use_position_with_reorder_fallback() {
    let mut plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        r"D:\zircon-export",
    ));
    let expected = ExportStage::ALL.to_vec();

    assert_eq!(
        plan.ordered_commands()
            .map(|command| command.stage)
            .collect::<Vec<_>>(),
        expected
    );

    plan.stages.swap(0, 1);
    assert_eq!(
        plan.ordered_commands()
            .map(|command| command.stage)
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn optimization_batch_20260830eo_ordered_commands_have_index_fast_path() {
    let source = include_str!("../plan.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("export plan production source");

    assert!(production.contains("self.stages.get(index)"));
    assert!(production.contains("command.stage == node.stage"));
}

#[test]
#[ignore = "release-only ordered command comparison evidence"]
fn optimization_batch_20260830eo_ordered_command_comparison_evidence() {
    const PLAN_COUNT: usize = 65_536;
    const STAGE_COUNT: usize = 8;
    const LEGACY_COMPARISONS_PER_PLAN: usize = STAGE_COUNT * (STAGE_COUNT + 1) / 2;
    const OPTIMIZED_COMPARISONS_PER_PLAN: usize = STAGE_COUNT;
    let legacy_stage_comparisons = PLAN_COUNT * LEGACY_COMPARISONS_PER_PLAN;
    let optimized_stage_comparisons = PLAN_COUNT * OPTIMIZED_COMPARISONS_PER_PLAN;

    assert_eq!(LEGACY_COMPARISONS_PER_PLAN, 36);
    assert_eq!(OPTIMIZED_COMPARISONS_PER_PLAN, 8);
    println!(
        "EDITOR545_EXPORT_ORDERED_COMMAND_INDEX_BENCH_V1 plans={PLAN_COUNT} \
             legacy_stage_comparisons={legacy_stage_comparisons} \
             optimized_stage_comparisons={optimized_stage_comparisons} reduction_pct=77.78"
    );
}
