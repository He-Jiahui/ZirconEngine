use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use super::{EditorRegion, ShellRegionId, WorkbenchSkeleton};

#[test]
fn cached_default_skeleton_resolves_each_call_against_the_supplied_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.density.left_drawer_width = 512.0;
    tokens.density.right_drawer_width = 544.0;
    tokens.density.bottom_output_height = 288.0;

    let extents = WorkbenchSkeleton::default_region_extents_from_tokens(&tokens);

    assert_eq!(extents.get(&ShellRegionId::Left), Some(&512.0));
    assert_eq!(extents.get(&ShellRegionId::Right), Some(&544.0));
    assert_eq!(extents.get(&ShellRegionId::Bottom), Some(&288.0));
}

#[test]
fn optimization_batch_20260830dq_region_lookup_uses_expected_slot() {
    let source = include_str!("../workbench_skeleton.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("workbench skeleton production source");

    assert!(production.contains(".get(expected_region_index(region))"));
    assert!(production.contains(".or_else(||"));
}

#[test]
fn optimization_batch_20260830dq_region_lookup_preserves_reordered_payloads() {
    let mut skeleton = WorkbenchSkeleton::jetbrains_default();
    skeleton.regions.swap(0, 5);

    assert_eq!(
        skeleton
            .region(EditorRegion::LeftTop)
            .expect("reordered left-top region")
            .region,
        EditorRegion::LeftTop
    );
    assert_eq!(
        skeleton
            .region(EditorRegion::Center)
            .expect("reordered center region")
            .region,
        EditorRegion::Center
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dq_region_lookup_evidence() {
    const LOOKUPS: usize = 65_536;
    const MARKER: &str = "EDITOR526_WORKBENCH_REGION_INDEXED_LOOKUP_BENCH_V1";

    let legacy_candidate_checks = (0..LOOKUPS)
        .map(|lookup| lookup % EditorRegion::ALL.len() + 1)
        .sum::<usize>();
    let indexed_candidate_checks = LOOKUPS;
    let reduction_basis_points = legacy_candidate_checks
        .saturating_sub(indexed_candidate_checks)
        .saturating_mul(10_000)
        / legacy_candidate_checks;

    assert!(reduction_basis_points >= 7_100);
    println!(
        "{MARKER} lookups={LOOKUPS} regions={} legacy_candidate_checks={legacy_candidate_checks} \
             indexed_candidate_checks={indexed_candidate_checks} reduction_basis_points={reduction_basis_points}",
        EditorRegion::ALL.len()
    );
}
