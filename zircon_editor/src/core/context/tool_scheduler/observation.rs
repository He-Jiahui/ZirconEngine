use crate::core::tools::{
    ToolAuthorityState, ToolOwnerGeneration, ToolQueueLimits, ToolResourceKindRegistration,
    ToolSchedulerStateSnapshot, ToolTransitionBatch, ToolTransitionRevision,
};

pub const DEFAULT_MAX_TOOL_TRANSITION_JOURNAL_BATCHES: usize = 256;
pub const DEFAULT_MAX_ACTIVE_TOOL_OWNER_GENERATIONS: usize = 1_024;

/// Hard limits for scheduler queues and the in-memory transition resync journal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolSchedulerLimits {
    queue_limits: ToolQueueLimits,
    max_transition_journal_batches: usize,
    max_active_owner_generations: usize,
}

impl ToolSchedulerLimits {
    pub fn new(
        queue_limits: ToolQueueLimits,
        max_transition_journal_batches: usize,
    ) -> Result<Self, ToolSchedulerLimitsError> {
        if max_transition_journal_batches == 0 {
            return Err(ToolSchedulerLimitsError::EmptyTransitionJournal);
        }
        Ok(Self {
            queue_limits,
            max_transition_journal_batches,
            max_active_owner_generations: DEFAULT_MAX_ACTIVE_TOOL_OWNER_GENERATIONS,
        })
    }

    pub fn with_owner_generation_capacity(
        queue_limits: ToolQueueLimits,
        max_transition_journal_batches: usize,
        max_active_owner_generations: usize,
    ) -> Result<Self, ToolSchedulerLimitsError> {
        let mut limits = Self::new(queue_limits, max_transition_journal_batches)?;
        if max_active_owner_generations == 0 {
            return Err(ToolSchedulerLimitsError::EmptyOwnerGenerationCapacity);
        }
        limits.max_active_owner_generations = max_active_owner_generations;
        Ok(limits)
    }

    pub const fn queue_limits(self) -> ToolQueueLimits {
        self.queue_limits
    }

    pub const fn max_transition_journal_batches(self) -> usize {
        self.max_transition_journal_batches
    }

    pub const fn max_active_owner_generations(self) -> usize {
        self.max_active_owner_generations
    }

    pub(super) const fn with_default_journal(queue_limits: ToolQueueLimits) -> Self {
        Self {
            queue_limits,
            max_transition_journal_batches: DEFAULT_MAX_TOOL_TRANSITION_JOURNAL_BATCHES,
            max_active_owner_generations: DEFAULT_MAX_ACTIVE_TOOL_OWNER_GENERATIONS,
        }
    }
}

impl Default for ToolSchedulerLimits {
    fn default() -> Self {
        Self::with_default_journal(ToolQueueLimits::new(
            crate::core::tools::DEFAULT_MAX_SINGLE_QUEUE_PER_RESOURCE,
            crate::core::tools::DEFAULT_MAX_SET_QUEUE,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolSchedulerLimitsError {
    EmptyTransitionJournal,
    EmptyOwnerGenerationCapacity,
}

impl std::fmt::Display for ToolSchedulerLimitsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTransitionJournal => {
                formatter.write_str("tool transition journal capacity must be nonzero")
            }
            Self::EmptyOwnerGenerationCapacity => {
                formatter.write_str("tool owner generation capacity must be nonzero")
            }
        }
    }
}

impl std::error::Error for ToolSchedulerLimitsError {}

/// Cursor naming the last tool transition revision observed by a consumer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolTransitionCursor(ToolTransitionRevision);

impl ToolTransitionCursor {
    pub const INITIAL: Self = Self(ToolTransitionRevision::INITIAL);

    pub const fn from_revision(revision: ToolTransitionRevision) -> Self {
        Self(revision)
    }

    pub const fn revision(self) -> ToolTransitionRevision {
        self.0
    }
}

impl From<ToolTransitionRevision> for ToolTransitionCursor {
    fn from(revision: ToolTransitionRevision) -> Self {
        Self::from_revision(revision)
    }
}

/// Atomic scheduler state captured at one committed transition revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolSchedulerSnapshot {
    cursor: ToolTransitionCursor,
    authority_state: ToolAuthorityState,
    active_owner_generations: Box<[ToolOwnerGeneration]>,
    resource_catalog: Box<[ToolResourceKindRegistration]>,
    state: ToolSchedulerStateSnapshot,
}

impl ToolSchedulerSnapshot {
    pub(super) fn new(
        revision: ToolTransitionRevision,
        authority_state: ToolAuthorityState,
        active_owner_generations: Box<[ToolOwnerGeneration]>,
        resource_catalog: Box<[ToolResourceKindRegistration]>,
        state: ToolSchedulerStateSnapshot,
    ) -> Self {
        Self {
            cursor: ToolTransitionCursor::from_revision(revision),
            authority_state,
            active_owner_generations,
            resource_catalog,
            state,
        }
    }

    pub const fn cursor(&self) -> ToolTransitionCursor {
        self.cursor
    }

    pub const fn authority_state(&self) -> ToolAuthorityState {
        self.authority_state
    }

    pub fn active_owner_generations(&self) -> &[ToolOwnerGeneration] {
        &self.active_owner_generations
    }

    pub fn resource_catalog(&self) -> &[ToolResourceKindRegistration] {
        &self.resource_catalog
    }

    pub fn state(&self) -> &ToolSchedulerStateSnapshot {
        &self.state
    }
}

/// Result of reading committed transitions after a consumer cursor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolTransitionRead {
    Current {
        cursor: ToolTransitionCursor,
    },
    Available {
        from_exclusive: ToolTransitionCursor,
        through: ToolTransitionCursor,
        batches: Box<[ToolTransitionBatch]>,
    },
    ResyncRequired {
        requested: ToolTransitionCursor,
        oldest_available_revision: ToolTransitionRevision,
        snapshot: ToolSchedulerSnapshot,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolTransitionReadError {
    FutureCursor {
        requested: ToolTransitionCursor,
        current: ToolTransitionCursor,
    },
}

impl std::fmt::Display for ToolTransitionReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FutureCursor { requested, current } => write!(
                formatter,
                "tool transition cursor {} is ahead of current revision {}",
                requested.revision().value(),
                current.revision().value()
            ),
        }
    }
}

impl std::error::Error for ToolTransitionReadError {}
