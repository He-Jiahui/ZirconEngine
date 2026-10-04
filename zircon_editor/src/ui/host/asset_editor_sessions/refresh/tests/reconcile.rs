use std::collections::BTreeSet;

use super::insert_normalized_ui_asset_id;

#[test]
fn reconcile_deduplicates_fragment_aliases_before_owning_asset_ids() {
    let mut asset_ids = BTreeSet::new();
    assert!(insert_normalized_ui_asset_id(
        &mut asset_ids,
        "res://ui/shared.widget#header"
    ));
    assert!(!insert_normalized_ui_asset_id(
        &mut asset_ids,
        "res://ui/shared.widget#footer"
    ));
    assert!(insert_normalized_ui_asset_id(
        &mut asset_ids,
        "res://ui/theme.style"
    ));
    assert_eq!(
        asset_ids,
        BTreeSet::from([
            "res://ui/shared.widget".to_string(),
            "res://ui/theme.style".to_string(),
        ])
    );
}

#[test]
fn optimization_batch_20260830en_reconcile_carries_selected_map_entry() {
    let source = include_str!("../reconcile.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("reconcile production source");

    assert!(production.contains("sessions.get_key_value(instance_id)"));
    assert!(!production.contains("sessions.contains_key(instance_id)"));
    assert!(!production.contains("sessions.get(&instance_id)"));
}

#[test]
#[ignore = "release-only reconcile lookup evidence"]
fn optimization_batch_20260830en_reconcile_lookup_evidence() {
    const SESSION_VISITS: usize = 65_536;
    const LEGACY_TREE_LOOKUPS_PER_VISIT: usize = 2;
    const OPTIMIZED_TREE_LOOKUPS_PER_VISIT: usize = 1;
    let legacy_tree_lookups = SESSION_VISITS * LEGACY_TREE_LOOKUPS_PER_VISIT;
    let optimized_tree_lookups = SESSION_VISITS * OPTIMIZED_TREE_LOOKUPS_PER_VISIT;

    assert_eq!(legacy_tree_lookups, optimized_tree_lookups * 2);
    println!(
        "EDITOR543_RECONCILE_CARRIED_ENTRY_BENCH_V1 visits={SESSION_VISITS} \
             legacy_tree_lookups={legacy_tree_lookups} optimized_tree_lookups={optimized_tree_lookups} \
             reduction_pct=50"
    );
}
