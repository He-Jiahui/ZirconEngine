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
        let selected_instance_id: Option<ViewInstanceId> = match cursor.current_instance_id.as_ref()
        {
            Some(instance_id) if sessions.contains_key(instance_id) => {
                Some(ViewInstanceId::clone(instance_id))
            }
            Some(instance_id) => {
                let after = ViewInstanceId::clone(instance_id);
                sessions
                    .range::<ViewInstanceId, _>((Excluded(after), Unbounded))
                    .next()
                    .map(|(next_id, _)| next_id.clone())
            }
            None => sessions
                .first_key_value()
                .map(|(first_id, _)| first_id.clone()),
        };
        let Some(instance_id) = selected_instance_id else {
            return (asset_ids, true);
        };
        if cursor.current_instance_id.as_ref() != Some(&instance_id) {
            cursor.current_instance_id = Some(instance_id.clone());
            cursor.next_item_index = 0;
        }

        let Some(entry) = sessions.get(&instance_id) else {
            cursor.current_instance_id = None;
            cursor.next_item_index = 0;
            continue;
        };
        let item_count = entry.session.import_reference_count().saturating_add(1);
        if cursor.next_item_index >= item_count {
            cursor.current_instance_id = sessions
                .range::<ViewInstanceId, _>((Excluded(instance_id), Unbounded))
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
