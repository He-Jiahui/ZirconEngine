use std::sync::atomic::Ordering;

use crate::core::framework::text::TextFontCollectionHandle;

use super::{
    TextRuntimeContext, TextRuntimeContextId, TextRuntimeContextLifecycleState,
    TextSystemFontPolicy,
};
use crate::text::UnicodeDataSnapshotId;

/// One immutable observation of a Runtime text capability root.
///
/// This health layer reports authority, cumulative admission, and active logical session families.
/// Request receipts, cache residency, workers, and frame correlation remain separate future
/// snapshots until their ownership is explicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRuntimeContextHealthSnapshot {
    context: TextRuntimeContextId,
    lifecycle: TextRuntimeContextLifecycleState,
    font_collection: TextFontCollectionHandle,
    font_generation: u64,
    system_font_policy: TextSystemFontPolicy,
    discovered_system_face_count: usize,
    unicode_data: UnicodeDataSnapshotId,
    admitted_layout_session_total: u64,
    active_layout_session_family_count: u64,
}

impl TextRuntimeContextHealthSnapshot {
    pub const fn context(self) -> TextRuntimeContextId {
        self.context
    }

    pub const fn lifecycle(self) -> TextRuntimeContextLifecycleState {
        self.lifecycle
    }

    pub const fn font_collection(self) -> TextFontCollectionHandle {
        self.font_collection
    }

    pub const fn font_generation(self) -> u64 {
        self.font_generation
    }

    pub const fn system_font_policy(self) -> TextSystemFontPolicy {
        self.system_font_policy
    }

    pub const fn discovered_system_face_count(self) -> usize {
        self.discovered_system_face_count
    }

    pub const fn unicode_data(self) -> UnicodeDataSnapshotId {
        self.unicode_data
    }

    pub const fn admitted_layout_session_total(self) -> u64 {
        self.admitted_layout_session_total
    }

    pub const fn active_layout_session_family_count(self) -> u64 {
        self.active_layout_session_family_count
    }
}

impl TextRuntimeContext {
    pub fn health_snapshot(&self) -> TextRuntimeContextHealthSnapshot {
        let font_revision = self.font_collection_revision();
        let (lifecycle, active_layout_session_family_count) = self.lifecycle.snapshot();
        TextRuntimeContextHealthSnapshot {
            context: self.id(),
            lifecycle,
            font_collection: font_revision.collection_id(),
            font_generation: font_revision.generation(),
            system_font_policy: self.system_font_policy,
            discovered_system_face_count: self.discovered_system_face_count,
            unicode_data: self.unicode_data_snapshot().id(),
            admitted_layout_session_total: self
                .admitted_layout_session_total
                .load(Ordering::Acquire),
            active_layout_session_family_count,
        }
    }
}

#[cfg(test)]
#[path = "tests/health.rs"]
mod tests;
