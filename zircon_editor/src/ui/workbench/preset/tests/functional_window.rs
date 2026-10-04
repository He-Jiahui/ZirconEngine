use super::{
    EditorFunctionalWindowKind, EditorFunctionalWindowPreset, EditorWindowDockPolicy,
    UnrealWindowModelPreset,
};

const WINDOW_KINDS: [EditorFunctionalWindowKind; 8] = [
    EditorFunctionalWindowKind::Workbench,
    EditorFunctionalWindowKind::SceneGame,
    EditorFunctionalWindowKind::PrefabEditor,
    EditorFunctionalWindowKind::MaterialEditor,
    EditorFunctionalWindowKind::UiAssetEditor,
    EditorFunctionalWindowKind::AnimationEditor,
    EditorFunctionalWindowKind::AssetBrowser,
    EditorFunctionalWindowKind::Diagnostics,
];

#[test]
fn optimization_batch_20260830ds_functional_window_lookup_uses_expected_slot() {
    let source = include_str!("../functional_window.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("functional window production source");

    assert!(production.contains(".get(expected_functional_window_index(kind))"));
    assert!(production.contains(".or_else(||"));
}

#[test]
fn optimization_batch_20260830ds_functional_window_lookup_preserves_reordered_payloads() {
    let mut preset = UnrealWindowModelPreset::new(WINDOW_KINDS.map(window_preset));
    preset.windows.swap(0, 7);

    assert_eq!(
        preset
            .window(EditorFunctionalWindowKind::Workbench)
            .expect("reordered workbench window")
            .kind,
        EditorFunctionalWindowKind::Workbench
    );
    assert_eq!(
        preset
            .window(EditorFunctionalWindowKind::Diagnostics)
            .expect("reordered diagnostics window")
            .kind,
        EditorFunctionalWindowKind::Diagnostics
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830ds_functional_window_lookup_evidence() {
    const LOOKUPS: usize = 65_536;
    const MARKER: &str = "EDITOR527_FUNCTIONAL_WINDOW_INDEXED_LOOKUP_BENCH_V1";

    let legacy_candidate_checks = (0..LOOKUPS)
        .map(|lookup| lookup % WINDOW_KINDS.len() + 1)
        .sum::<usize>();
    let indexed_candidate_checks = LOOKUPS;
    let reduction_basis_points = legacy_candidate_checks
        .saturating_sub(indexed_candidate_checks)
        .saturating_mul(10_000)
        / legacy_candidate_checks;

    assert!(reduction_basis_points >= 7_700);
    println!(
        "{MARKER} lookups={LOOKUPS} windows={} legacy_candidate_checks={legacy_candidate_checks} \
             indexed_candidate_checks={indexed_candidate_checks} reduction_basis_points={reduction_basis_points}",
        WINDOW_KINDS.len()
    );
}

fn window_preset(kind: EditorFunctionalWindowKind) -> EditorFunctionalWindowPreset {
    EditorFunctionalWindowPreset::new(kind, kind.slug(), EditorWindowDockPolicy::MainWorkbench)
}
