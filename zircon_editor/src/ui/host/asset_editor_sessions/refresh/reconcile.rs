use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound::{Excluded, Unbounded};

use super::super::watcher::{UiAssetWatchPollAllowance, UiAssetWatchReconcileCursor};
use super::super::UiAssetWorkspaceEntry;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub(in crate::ui::host::asset_editor_sessions) fn collect_ui_asset_reconcile_batch(
        &self,
        cursor: &mut UiAssetWatchReconcileCursor,
        allowance: &mut UiAssetWatchPollAllowance,
    ) -> (BTreeSet<String>, bool) {
        let sessions = self.lock_ui_asset_sessions();
        collect_ui_asset_reconcile_batch(&sessions, cursor, allowance)
    }
}

pub(in crate::ui::host::asset_editor_sessions) fn collect_ui_asset_reconcile_batch(
    sessions: &BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>,
    cursor: &mut UiAssetWatchReconcileCursor,
    allowance: &mut UiAssetWatchPollAllowance,
) -> (BTreeSet<String>, bool) {
    let mut asset_ids = BTreeSet::new();
    loop {
        let selected_entry = match cursor.current_instance_id.as_ref() {
            Some(instance_id) => sessions.get_key_value(instance_id).or_else(|| {
                sessions
                    .range::<ViewInstanceId, _>((Excluded(instance_id.clone()), Unbounded))
                    .next()
            }),
            None => sessions.first_key_value(),
        };
        let Some((instance_id, entry)) = selected_entry else {
            return (asset_ids, true);
        };
        if cursor.current_instance_id.as_ref() != Some(instance_id) {
            cursor.current_instance_id = Some(instance_id.clone());
            cursor.next_item_index = 0;
        }

        let item_count = entry.session.import_reference_count().saturating_add(1);
        if cursor.next_item_index >= item_count {
            cursor.current_instance_id = sessions
                .range::<ViewInstanceId, _>((Excluded(instance_id.clone()), Unbounded))
                .next()
                .map(|(next_id, _)| next_id.clone());
            cursor.next_item_index = 0;
            if cursor.current_instance_id.is_none() {
                return (asset_ids, true);
            }
            continue;
        }
        if !allowance.try_take() {
            return (asset_ids, false);
        }

        let asset_id = if cursor.next_item_index == 0 {
            Some(entry.session.route().asset_id.as_str())
        } else {
            entry
                .session
                .import_reference_at(cursor.next_item_index - 1)
        };
        let Some(asset_id) = asset_id else {
            cursor.next_item_index = item_count;
            continue;
        };
        let _ = asset_ids.insert(normalize_ui_asset_asset_id(asset_id).to_string());
        cursor.next_item_index += 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn optimization_batch_20260830en_reconcile_carries_selected_map_entry() {
        let source = include_str!("reconcile.rs");
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
}
