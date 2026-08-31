use std::time::Duration;

use super::{AutosaveDocumentId, AutosaveError};
use crate::core::editing::engine::HistoryDirtyState;
use crate::core::jobs::{EditorJobSpec, JobCategory, JobPriority, MutexGroup};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutosavePolicy {
    interval: Duration,
}

impl AutosavePolicy {
    pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(300);

    pub fn new(interval: Duration) -> Result<Self, AutosaveError> {
        if interval.is_zero() {
            return Err(AutosaveError::ZeroInterval);
        }
        Ok(Self { interval })
    }

    pub const fn interval(self) -> Duration {
        self.interval
    }
}

impl Default for AutosavePolicy {
    fn default() -> Self {
        Self {
            interval: Self::DEFAULT_INTERVAL,
        }
    }
}

/// Immutable scheduling constraints for an autosave task.
///
/// The save owner supplies its mutex group so autosaves can never overlap a
/// foreground save of the same document. The recovery core deliberately does
/// not submit jobs itself; Editor14 owns queue admission and execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveJobPolicy {
    save_mutex_group: MutexGroup,
}

impl AutosaveJobPolicy {
    pub fn for_save_mutex(save_mutex_group: MutexGroup) -> Self {
        Self { save_mutex_group }
    }

    pub const fn category(&self) -> JobCategory {
        JobCategory::Misc
    }

    pub const fn priority(&self) -> JobPriority {
        JobPriority::Background
    }

    pub fn save_mutex_group(&self) -> &MutexGroup {
        &self.save_mutex_group
    }

    pub fn build_job_spec(&self, document: &AutosaveDocumentId) -> EditorJobSpec {
        EditorJobSpec::new(format!("autosave:{}", document.as_str()), self.category())
            .with_priority(self.priority())
            .with_mutex_group(self.save_mutex_group.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveDocumentState {
    document: AutosaveDocumentId,
    dirty: bool,
}

impl AutosaveDocumentState {
    /// Projects Editor03's saved-top dirty fact for one document.
    ///
    /// Document-to-history routing belongs to the editor manager. Autosave
    /// receives only this immutable query result and never owns dirty state.
    pub fn from_history_dirty(document: AutosaveDocumentId, state: HistoryDirtyState) -> Self {
        Self {
            document,
            dirty: state.is_dirty(),
        }
    }

    pub(crate) const fn from_dirty_projection(document: AutosaveDocumentId, dirty: bool) -> Self {
        Self { document, dirty }
    }

    #[cfg(test)]
    pub(crate) fn from_dirty_for_test(document: AutosaveDocumentId, dirty: bool) -> Self {
        Self { document, dirty }
    }

    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosavePlan {
    pub(super) documents: Vec<AutosaveDocumentId>,
}

impl AutosavePlan {
    pub fn documents(&self) -> &[AutosaveDocumentId] {
        &self.documents
    }
}
