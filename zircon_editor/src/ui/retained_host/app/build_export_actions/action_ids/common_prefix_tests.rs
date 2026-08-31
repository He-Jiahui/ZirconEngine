use std::hint::black_box;
use std::time::Instant;

use super::{parse_build_export_action, BuildExportAction};

const ACTION_BYTES: usize = 512;
const CHECKS_PER_SAMPLE: usize = 131_072;
const SAMPLE_PAIRS: usize = 31;

fn legacy_parse_build_export_action(action_id: &str) -> Option<BuildExportAction<'_>> {
    if let Some(profile_name) = action_id
        .strip_prefix("workbench.build_export.plan.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::GeneratePlan { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("workbench.build_export.execute.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::Execute { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("workbench.build_export.cancel.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::Cancel { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("workbench.build_export.output.clear.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::ClearOutput { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("workbench.build_export.output.reveal.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::RevealOutput { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("workbench.build_export.output.choose.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::ChooseOutput { profile_name });
    }
    action_id
        .strip_prefix("workbench.build_export.output.set.")
        .and_then(|rest| rest.split_once('|'))
        .and_then(|(profile_name, output_root)| {
            if profile_name.trim().is_empty() || output_root.trim().is_empty() {
                None
            } else {
                Some(BuildExportAction::SetOutput {
                    profile_name,
                    output_root,
                })
            }
        })
}

fn projection<'a>(
    action: Option<BuildExportAction<'a>>,
) -> Option<(&'static str, &'a str, Option<&'a str>)> {
    action.map(|action| match action {
        BuildExportAction::GeneratePlan { profile_name } => ("plan", profile_name, None),
        BuildExportAction::Execute { profile_name } => ("execute", profile_name, None),
        BuildExportAction::Cancel { profile_name } => ("cancel", profile_name, None),
        BuildExportAction::SetOutput {
            profile_name,
            output_root,
        } => ("set", profile_name, Some(output_root)),
        BuildExportAction::ChooseOutput { profile_name } => ("choose", profile_name, None),
        BuildExportAction::ClearOutput { profile_name } => ("clear", profile_name, None),
        BuildExportAction::RevealOutput { profile_name } => ("reveal", profile_name, None),
    })
}

fn measure(action_id: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut matches = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let action = if optimized {
            parse_build_export_action(black_box(action_id))
        } else {
            legacy_parse_build_export_action(black_box(action_id))
        };
        matches += usize::from(matches!(action, Some(BuildExportAction::SetOutput { .. })));
    }
    black_box(matches);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn optimization_batch_20260829bb_editor274_common_prefix_parser_preserves_actions() {
    for action_id in [
        "workbench.build_export.plan.desktop",
        "workbench.build_export.execute.desktop",
        "workbench.build_export.cancel.desktop",
        "workbench.build_export.output.clear.desktop",
        "workbench.build_export.output.reveal.desktop",
        "workbench.build_export.output.choose.desktop",
        "workbench.build_export.output.set.desktop|D:/Builds",
        "workbench.build_export.execute.",
        "workbench.build_export.unknown.desktop",
        "other.build_export.plan.desktop",
    ] {
        assert_eq!(
            projection(parse_build_export_action(action_id)),
            projection(legacy_parse_build_export_action(action_id)),
            "{action_id:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bb_editor274_build_export_parser_strips_common_prefix_once() {
    let source = include_str!("../action_ids.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(production.matches("workbench.build_export.").count(), 1);
    assert!(production.contains("let action = action_id.strip_prefix"));
    assert!(production.contains("action\n        .strip_prefix(\"output.set.\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bb_editor274_common_prefix_build_export_actions_bench() {
    let profile_bytes = ACTION_BYTES - "workbench.build_export.output.set.|D:/Builds".len();
    let action_id = format!(
        "workbench.build_export.output.set.{}|D:/Builds",
        "p".repeat(profile_bytes)
    );
    assert_eq!(action_id.len(), ACTION_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&action_id, false));
            optimized_samples.push(measure(&action_id, true));
        } else {
            optimized_samples.push(measure(&action_id, true));
            legacy_samples.push(measure(&action_id, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR274_COMMON_PREFIX_BUILD_EXPORT_ACTIONS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} action_bytes={ACTION_BYTES} \
legacy_common_prefix_checks=7 optimized_common_prefix_checks=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
