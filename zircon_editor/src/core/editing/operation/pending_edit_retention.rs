use std::num::NonZeroUsize;
use std::time::Duration;

use crate::core::editor_operation::EditorOperationInvocation;

/// Deferred-edit retention is declared by the operation registration, never by a Play caller.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PendingEditRetention {
    #[default]
    Lossless,
    Latest,
    Bounded(PendingEditBounds),
}

impl PendingEditRetention {
    pub const fn latest() -> Self {
        Self::Latest
    }

    pub fn bounded(
        max_entries: usize,
        max_payload_bytes: usize,
        max_age: Duration,
    ) -> Result<Self, PendingEditRetentionError> {
        let Some(max_entries) = NonZeroUsize::new(max_entries) else {
            return Err(PendingEditRetentionError::ZeroBoundedEntries);
        };
        let Some(max_payload_bytes) = NonZeroUsize::new(max_payload_bytes) else {
            return Err(PendingEditRetentionError::ZeroBoundedPayloadBytes);
        };
        if max_age.is_zero() {
            return Err(PendingEditRetentionError::ZeroBoundedAge);
        }
        Ok(Self::Bounded(PendingEditBounds {
            max_entries,
            max_payload_bytes,
            max_age,
        }))
    }

    pub(crate) const fn is_latest(&self) -> bool {
        matches!(self, Self::Latest)
    }

    pub(crate) const fn bounded_limits(&self) -> Option<PendingEditBounds> {
        match self {
            Self::Bounded(limits) => Some(*limits),
            Self::Lossless | Self::Latest => None,
        }
    }

    pub(crate) const fn cohort_kind(&self) -> Option<PendingEditCohortKind> {
        match self {
            Self::Lossless => None,
            Self::Latest => Some(PendingEditCohortKind::Latest),
            Self::Bounded(_) => Some(PendingEditCohortKind::Bounded),
        }
    }
}

/// A bounded policy applies to one typed `(target, operation)` cohort in the Play queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingEditBounds {
    max_entries: NonZeroUsize,
    max_payload_bytes: NonZeroUsize,
    max_age: Duration,
}

impl PendingEditBounds {
    pub const fn max_entries(self) -> NonZeroUsize {
        self.max_entries
    }

    pub const fn max_payload_bytes(self) -> NonZeroUsize {
        self.max_payload_bytes
    }

    pub const fn max_age(self) -> Duration {
        self.max_age
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PendingEditCohortKind {
    Latest,
    Bounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingEditRetentionError {
    ZeroBoundedEntries,
    ZeroBoundedPayloadBytes,
    ZeroBoundedAge,
}

impl std::fmt::Display for PendingEditRetentionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroBoundedEntries => formatter
                .write_str("a bounded pending edit policy requires entries greater than zero"),
            Self::ZeroBoundedPayloadBytes => formatter.write_str(
                "a bounded pending edit policy requires payload bytes greater than zero",
            ),
            Self::ZeroBoundedAge => formatter
                .write_str("a bounded pending edit policy requires an age greater than zero"),
        }
    }
}

impl std::error::Error for PendingEditRetentionError {}

/// A policy-bound invocation that may enter a deferred Play edit queue.
#[derive(Clone, Debug, PartialEq)]
pub struct DeferredOperationInvocation {
    invocation: EditorOperationInvocation,
    retention: PendingEditRetention,
}

impl DeferredOperationInvocation {
    pub(crate) fn from_registration(
        invocation: EditorOperationInvocation,
        retention: PendingEditRetention,
    ) -> Self {
        Self {
            invocation,
            retention,
        }
    }

    pub fn invocation(&self) -> &EditorOperationInvocation {
        &self.invocation
    }

    pub fn retention(&self) -> &PendingEditRetention {
        &self.retention
    }

    pub(crate) fn into_parts(self) -> (EditorOperationInvocation, PendingEditRetention) {
        (self.invocation, self.retention)
    }
}
