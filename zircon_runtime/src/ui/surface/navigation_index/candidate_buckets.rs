//! 重建按清空标记、重新填充、删除未见项的顺序复用候选容器；清空内容不释放稳定 scope 的 Vec 容量。
use std::collections::{BTreeMap, HashMap};

use zircon_runtime_interface::ui::{event_ui::UiNodeId, navigation::UiNavigationGroupId};

#[derive(Clone, Debug)]
pub(super) struct FirstGroupCandidate {
    pub(super) node_id: UiNodeId,
    pub(super) seen: bool,
}

pub(super) fn reset_first_group_candidates(
    candidates: &mut HashMap<UiNavigationGroupId, FirstGroupCandidate>,
) {
    for candidate in candidates.values_mut() {
        candidate.seen = false;
    }
}

pub(super) fn prune_first_group_candidates(
    candidates: &mut HashMap<UiNavigationGroupId, FirstGroupCandidate>,
) {
    candidates.retain(|_, candidate| candidate.seen);
}

pub(super) fn clear_candidate_buckets<K: Ord>(candidates: &mut BTreeMap<K, Vec<UiNodeId>>) {
    for bucket in candidates.values_mut() {
        bucket.clear();
    }
}

pub(super) fn prune_empty_candidate_buckets<K: Ord>(candidates: &mut BTreeMap<K, Vec<UiNodeId>>) {
    candidates.retain(|_, bucket| !bucket.is_empty());
}

pub(super) fn push_group_candidate(
    candidates: &mut BTreeMap<UiNavigationGroupId, Vec<UiNodeId>>,
    group_id: &UiNavigationGroupId,
    node_id: UiNodeId,
) {
    if let Some(bucket) = candidates.get_mut(group_id) {
        bucket.push(node_id);
        return;
    }
    candidates.insert(group_id.clone(), vec![node_id]);
}
