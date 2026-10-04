//! 把脏文档按稳定身份选入单次自动保存窗口；常规周期与关闭时最终窗口共享单飞门，提交失败可释放窗口重试。

use std::collections::BTreeSet;
use std::time::Duration;

use super::{AutosaveDocumentId, AutosaveDocumentState, AutosavePlan, AutosavePolicy};

#[derive(Clone, Debug)]
pub struct AutosaveScheduler {
    policy: AutosavePolicy,
    next_due_at: Duration,
    in_flight: bool,
}

impl AutosaveScheduler {
    pub fn new(policy: AutosavePolicy) -> Self {
        Self {
            next_due_at: policy.interval(),
            policy,
            in_flight: false,
        }
    }

    pub fn plan(
        &mut self,
        now: Duration,
        documents: &[AutosaveDocumentState],
    ) -> Option<AutosavePlan> {
        self.plan_window(now, documents, usize::MAX, None)
    }

    /// Applies a validated cadence without resetting the current autosave window.
    pub(in crate::core::recovery) fn update_policy(&mut self, policy: AutosavePolicy) {
        if self.policy == policy {
            return;
        }
        let anchor = self.next_due_at.saturating_sub(self.policy.interval());
        self.policy = policy;
        self.next_due_at = anchor
            .checked_add(self.policy.interval())
            .unwrap_or(Duration::MAX);
    }

    pub(in crate::core::recovery) fn is_due(&self, now: Duration) -> bool {
        !self.in_flight && now >= self.next_due_at
    }

    pub(in crate::core::recovery) fn plan_window(
        &mut self,
        now: Duration,
        documents: &[AutosaveDocumentState],
        max_documents: usize,
        start_after: Option<&AutosaveDocumentId>,
    ) -> Option<AutosavePlan> {
        if !self.is_due(now) {
            return None;
        }
        self.plan_window_without_deadline(documents, max_documents, start_after)
    }

    /// Creates one final shutdown window without waiting for the periodic
    /// deadline. The caller must have already fenced regular autosave
    /// admission, and this still preserves the single-flight invariant.
    /// 关闭流程先阻断常规准入才可跳过周期截止；仍受单飞门保护，避免与在途写入重叠。
    pub(in crate::core::recovery) fn plan_final_window(
        &mut self,
        documents: &[AutosaveDocumentState],
        max_documents: usize,
        start_after: Option<&AutosaveDocumentId>,
    ) -> Option<AutosavePlan> {
        if self.in_flight {
            return None;
        }
        self.plan_window_without_deadline(documents, max_documents, start_after)
    }

    fn plan_window_without_deadline(
        &mut self,
        documents: &[AutosaveDocumentState],
        max_documents: usize,
        start_after: Option<&AutosaveDocumentId>,
    ) -> Option<AutosavePlan> {
        if max_documents == 0 {
            return None;
        }
        let mut after_cursor = BTreeSet::new();
        let mut wrapped = BTreeSet::new();
        for state in documents.iter().filter(|state| state.is_dirty()) {
            let document = state.document();
            if start_after.is_some_and(|cursor| document <= cursor) {
                insert_bounded_document(&mut wrapped, document.clone(), max_documents);
            } else {
                insert_bounded_document(&mut after_cursor, document.clone(), max_documents);
            }
        }
        let mut documents = after_cursor.into_iter().collect::<Vec<_>>();
        documents.extend(
            wrapped
                .into_iter()
                .take(max_documents.saturating_sub(documents.len())),
        );
        if documents.is_empty() {
            return None;
        }
        self.in_flight = true;
        Some(AutosavePlan { documents })
    }

    /// Completes an admitted autosave job, regardless of its write result.
    pub fn mark_finished(&mut self, at: Duration) {
        self.in_flight = false;
        self.next_due_at = at
            .checked_add(self.policy.interval())
            .unwrap_or(Duration::MAX);
    }

    /// Releases a plan whose job was not admitted, so the caller can retry it.
    pub fn mark_submission_failed(&mut self) {
        self.in_flight = false;
    }

    pub const fn is_in_flight(&self) -> bool {
        self.in_flight
    }
}

fn insert_bounded_document(
    documents: &mut BTreeSet<AutosaveDocumentId>,
    document: AutosaveDocumentId,
    max_documents: usize,
) {
    documents.insert(document);
    if documents.len() > max_documents {
        documents.pop_last();
    }
}

#[cfg(test)]
#[path = "tests/scheduler.rs"]
mod tests;
