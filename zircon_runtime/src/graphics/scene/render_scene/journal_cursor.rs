use std::error::Error;
use std::fmt;

use crate::core::framework::render::RenderWorldSnapshotHandle;

use super::change_journal::RenderSceneChangeJournal;
use super::scene::RenderSceneGeneration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderSceneJournalCursor {
    world: RenderWorldSnapshotHandle,
    applied_generation: RenderSceneGeneration,
}

impl RenderSceneJournalCursor {
    pub(crate) const fn at(
        world: RenderWorldSnapshotHandle,
        applied_generation: RenderSceneGeneration,
    ) -> Self {
        Self {
            world,
            applied_generation,
        }
    }

    pub(crate) const fn world(&self) -> RenderWorldSnapshotHandle {
        self.world
    }

    pub(crate) const fn applied_generation(&self) -> RenderSceneGeneration {
        self.applied_generation
    }

    pub(crate) fn preflight(
        &self,
        journal: &RenderSceneChangeJournal,
    ) -> Result<RenderSceneJournalPreflight, RenderSceneJournalCursorError> {
        let journal_from_generation = journal.from_generation();
        let journal_to_generation = journal.to_generation();
        if journal.world() != self.world {
            return Err(RenderSceneJournalCursorError::WorldChanged {
                expected_world: self.world,
                journal_world: journal.world(),
            });
        }
        validate_generation_range(journal_from_generation, journal_to_generation)?;
        if journal_to_generation < self.applied_generation {
            return Err(RenderSceneJournalCursorError::StaleJournal {
                applied_generation: self.applied_generation,
                journal_to_generation,
            });
        }
        if journal_to_generation == self.applied_generation {
            return Ok(RenderSceneJournalPreflight::replay(
                self.world,
                journal_from_generation,
                journal_to_generation,
            ));
        }
        if journal_from_generation != self.applied_generation {
            return Err(RenderSceneJournalCursorError::GenerationGap {
                applied_generation: self.applied_generation,
                journal_from_generation,
                journal_to_generation,
            });
        }
        Ok(RenderSceneJournalPreflight::apply(
            self.world,
            journal_from_generation,
            journal_to_generation,
        ))
    }

    pub(crate) fn commit(
        &mut self,
        preflight: RenderSceneJournalPreflight,
    ) -> Result<RenderSceneJournalCommit, RenderSceneJournalCursorError> {
        if preflight.world() != self.world {
            return Err(RenderSceneJournalCursorError::WorldChanged {
                expected_world: self.world,
                journal_world: preflight.world(),
            });
        }
        validate_generation_range(
            preflight.journal_from_generation(),
            preflight.journal_to_generation(),
        )?;
        let expected_generation = preflight.expected_applied_generation();
        if self.applied_generation != expected_generation {
            return Err(RenderSceneJournalCursorError::CursorAdvanced {
                expected_generation,
                applied_generation: self.applied_generation,
            });
        }
        match preflight.kind {
            RenderSceneJournalPreflightKind::Replay => Ok(RenderSceneJournalCommit::Replayed),
            RenderSceneJournalPreflightKind::Apply => {
                self.applied_generation = preflight.journal_to_generation;
                Ok(RenderSceneJournalCommit::Applied)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderSceneJournalPreflight {
    kind: RenderSceneJournalPreflightKind,
    world: RenderWorldSnapshotHandle,
    journal_from_generation: RenderSceneGeneration,
    journal_to_generation: RenderSceneGeneration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderSceneJournalPreflightKind {
    Replay,
    Apply,
}

impl RenderSceneJournalPreflight {
    const fn replay(
        world: RenderWorldSnapshotHandle,
        journal_from_generation: RenderSceneGeneration,
        journal_to_generation: RenderSceneGeneration,
    ) -> Self {
        Self {
            kind: RenderSceneJournalPreflightKind::Replay,
            world,
            journal_from_generation,
            journal_to_generation,
        }
    }

    const fn apply(
        world: RenderWorldSnapshotHandle,
        journal_from_generation: RenderSceneGeneration,
        journal_to_generation: RenderSceneGeneration,
    ) -> Self {
        Self {
            kind: RenderSceneJournalPreflightKind::Apply,
            world,
            journal_from_generation,
            journal_to_generation,
        }
    }

    pub(crate) const fn requires_apply(self) -> bool {
        matches!(self.kind, RenderSceneJournalPreflightKind::Apply)
    }

    pub(crate) const fn journal_from_generation(self) -> RenderSceneGeneration {
        self.journal_from_generation
    }

    pub(crate) const fn world(self) -> RenderWorldSnapshotHandle {
        self.world
    }

    pub(crate) const fn journal_to_generation(self) -> RenderSceneGeneration {
        self.journal_to_generation
    }

    const fn expected_applied_generation(self) -> RenderSceneGeneration {
        match self.kind {
            RenderSceneJournalPreflightKind::Replay => self.journal_to_generation,
            RenderSceneJournalPreflightKind::Apply => self.journal_from_generation,
        }
    }
}

fn validate_generation_range(
    journal_from_generation: RenderSceneGeneration,
    journal_to_generation: RenderSceneGeneration,
) -> Result<(), RenderSceneJournalCursorError> {
    if journal_to_generation < journal_from_generation {
        return Err(RenderSceneJournalCursorError::InvalidJournalRange {
            journal_from_generation,
            journal_to_generation,
        });
    }
    if journal_to_generation
        .get()
        .saturating_sub(journal_from_generation.get())
        > 1
    {
        return Err(RenderSceneJournalCursorError::NonAdjacentJournalRange {
            journal_from_generation,
            journal_to_generation,
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneJournalCommit {
    Applied,
    Replayed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneJournalCursorError {
    WorldChanged {
        expected_world: RenderWorldSnapshotHandle,
        journal_world: RenderWorldSnapshotHandle,
    },
    InvalidJournalRange {
        journal_from_generation: RenderSceneGeneration,
        journal_to_generation: RenderSceneGeneration,
    },
    NonAdjacentJournalRange {
        journal_from_generation: RenderSceneGeneration,
        journal_to_generation: RenderSceneGeneration,
    },
    StaleJournal {
        applied_generation: RenderSceneGeneration,
        journal_to_generation: RenderSceneGeneration,
    },
    GenerationGap {
        applied_generation: RenderSceneGeneration,
        journal_from_generation: RenderSceneGeneration,
        journal_to_generation: RenderSceneGeneration,
    },
    CursorAdvanced {
        expected_generation: RenderSceneGeneration,
        applied_generation: RenderSceneGeneration,
    },
}

impl fmt::Display for RenderSceneJournalCursorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorldChanged {
                expected_world,
                journal_world,
            } => write!(
                formatter,
                "render-scene consumer for world {} cannot apply journal from world {}",
                expected_world.raw(),
                journal_world.raw()
            ),
            Self::InvalidJournalRange {
                journal_from_generation,
                journal_to_generation,
            } => write!(
                formatter,
                "render-scene journal range {}..{} is inverted",
                journal_from_generation.get(),
                journal_to_generation.get()
            ),
            Self::NonAdjacentJournalRange {
                journal_from_generation,
                journal_to_generation,
            } => write!(
                formatter,
                "render-scene journal range {}..{} spans more than one generation",
                journal_from_generation.get(),
                journal_to_generation.get()
            ),
            Self::StaleJournal {
                applied_generation,
                journal_to_generation,
            } => write!(
                formatter,
                "render-scene journal ending at generation {} is older than consumer generation {}",
                journal_to_generation.get(),
                applied_generation.get()
            ),
            Self::GenerationGap {
                applied_generation,
                journal_from_generation,
                journal_to_generation,
            } => write!(
                formatter,
                "render-scene consumer generation {} cannot apply journal {}..{}",
                applied_generation.get(),
                journal_from_generation.get(),
                journal_to_generation.get()
            ),
            Self::CursorAdvanced {
                expected_generation,
                applied_generation,
            } => write!(
                formatter,
                "render-scene consumer advanced from expected generation {} to {} before commit",
                expected_generation.get(),
                applied_generation.get()
            ),
        }
    }
}

impl Error for RenderSceneJournalCursorError {}

#[cfg(test)]
mod tests;
