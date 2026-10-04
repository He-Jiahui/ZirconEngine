//! 不可变发布中的清单路径索引保留重复包的全部候选，使删除当前赢家后可确定性提升下一候选。
//! 增量操作应用在基准索引的副本上；失败副本不会替换消费者仍持有的快照。
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::super::NativePluginCandidate;
use super::{
    NativePluginDiscoveryManifestAction, NativePluginDiscoveryRefreshError,
    NativePluginDiscoveryRefreshWork,
};

#[cfg(test)]
#[path = "manifest_index/tests/capacity_tests.rs"]
mod capacity_tests;

/// Immutable manifest-path truth used to project deterministic candidates and duplicate reports.
#[derive(Clone, Debug, Default)]
pub(super) struct NativePluginDiscoveryManifestIndex {
    candidates: BTreeMap<PathBuf, NativePluginCandidate>,
}

impl NativePluginDiscoveryManifestIndex {
    pub(super) fn from_candidates(candidates: Vec<NativePluginCandidate>) -> Self {
        let mut index = Self::default();
        for candidate in candidates {
            index
                .candidates
                .insert(candidate.manifest_path.clone(), candidate);
        }
        index
    }

    /// 工作动作与候选须来自同一通知批次；按通知顺序执行父目录删除与子清单重建。
    /// 调用者必须使用未发布的副本，因为报错前可能已修改此索引。
    pub(super) fn apply_incremental(
        &mut self,
        work: &NativePluginDiscoveryRefreshWork,
        refreshed_candidates: &[NativePluginCandidate],
        max_candidates: usize,
    ) -> Result<(), NativePluginDiscoveryRefreshError> {
        let Some(actions) = work.manifest_actions() else {
            return Err(NativePluginDiscoveryRefreshError::collector(
                "full discovery work cannot be applied to an incremental manifest index",
            ));
        };
        let Some(ordered_paths) = work.manifest_paths_in_notification_order() else {
            return Err(NativePluginDiscoveryRefreshError::collector(
                "incremental native plugin discovery lost its notification order",
            ));
        };
        let mut refreshed = BTreeMap::new();
        for candidate in refreshed_candidates {
            refreshed.insert(candidate.manifest_path.clone(), candidate);
        }

        for path in ordered_paths {
            let action = actions
                .get(path)
                .expect("notification path must have an action");
            match action {
                NativePluginDiscoveryManifestAction::Refresh => {
                    let Some(candidate) = refreshed.remove(path) else {
                        return Err(NativePluginDiscoveryRefreshError::collector(format!(
                            "incremental native plugin discovery did not collect {}",
                            path.display()
                        )));
                    };
                    self.candidates.insert(path.clone(), candidate.clone());
                }
                NativePluginDiscoveryManifestAction::Remove => {
                    self.candidates
                        .retain(|manifest_path, _| !is_path_within(manifest_path, path));
                }
            }
        }
        if !refreshed.is_empty() {
            return Err(NativePluginDiscoveryRefreshError::collector(
                "incremental native plugin discovery collected a path outside its notification batch",
            ));
        }
        if self.candidates.len() > max_candidates {
            return Err(NativePluginDiscoveryRefreshError::budget_exceeded(
                super::NativePluginDiscoveryRefreshBudgetKind::CandidateCount,
                self.candidates.len() as u64,
                max_candidates as u64,
            ));
        }
        Ok(())
    }

    /// 清单路径排序决定同包标识的唯一赢家；重复项留在索引中，诊断随每次投影重算。
    pub(super) fn project(&self) -> (Vec<NativePluginCandidate>, Vec<String>) {
        let mut selected = BTreeMap::<String, PathBuf>::new();
        let mut candidates = Vec::with_capacity(self.candidates.len());
        let mut diagnostics = Vec::new();

        for (manifest_path, candidate) in &self.candidates {
            if let Some(first_path) = selected.get(&candidate.plugin_id) {
                // BUG: [CR-PLUGIN-NATIVE-0206] 这些重复包诊断未经过收集槽准入，也未校验最终诊断数；
                // 候选总数在额度内但重复项超过诊断上限时，发布快照仍越过声明的诊断预算；证据：快照直接接收本投影。
                diagnostics.push(format!(
                    "duplicate native plugin package id `{}`: keeping {}, ignoring {}",
                    candidate.plugin_id,
                    first_path.display(),
                    manifest_path.display()
                ));
                continue;
            }
            selected.insert(candidate.plugin_id.clone(), manifest_path.clone());
            candidates.push(candidate.clone());
        }
        (candidates, diagnostics)
    }
}

fn is_path_within(candidate_path: &Path, removed_path: &Path) -> bool {
    candidate_path == removed_path || candidate_path.starts_with(removed_path)
}
