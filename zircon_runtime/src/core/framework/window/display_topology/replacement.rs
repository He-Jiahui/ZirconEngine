use std::error::Error;
use std::fmt;

use super::{DisplayId, DisplayTopologyGeneration, DisplayTopologySnapshot};

/// A deterministic summary of one valid display-topology replacement.
///
/// Added and changed IDs retain the new snapshot's display order. Removed IDs
/// retain the previous snapshot's order. This keeps platform notifications and
/// diagnostics stable without treating an incidental backend index as identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayTopologyReplacement {
    previous_generation: DisplayTopologyGeneration,
    current_generation: DisplayTopologyGeneration,
    added: Vec<DisplayId>,
    removed: Vec<DisplayId>,
    changed: Vec<DisplayId>,
    primary_changed: bool,
}

impl DisplayTopologyReplacement {
    fn new(
        previous_generation: DisplayTopologyGeneration,
        current_generation: DisplayTopologyGeneration,
        added: Vec<DisplayId>,
        removed: Vec<DisplayId>,
        changed: Vec<DisplayId>,
        primary_changed: bool,
    ) -> Self {
        Self {
            previous_generation,
            current_generation,
            added,
            removed,
            changed,
            primary_changed,
        }
    }

    pub const fn previous_generation(&self) -> DisplayTopologyGeneration {
        self.previous_generation
    }

    pub const fn current_generation(&self) -> DisplayTopologyGeneration {
        self.current_generation
    }

    pub fn added(&self) -> &[DisplayId] {
        &self.added
    }

    pub fn removed(&self) -> &[DisplayId] {
        &self.removed
    }

    pub fn changed(&self) -> &[DisplayId] {
        &self.changed
    }

    pub const fn primary_changed(&self) -> bool {
        self.primary_changed
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && !self.primary_changed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayTopologyReplacementError {
    GenerationNotAdvanced {
        previous: DisplayTopologyGeneration,
        current: DisplayTopologyGeneration,
    },
    CapacityExhausted,
}

impl fmt::Display for DisplayTopologyReplacementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenerationNotAdvanced { previous, current } => write!(
                formatter,
                "display topology generation {} cannot replace generation {}",
                current.get(),
                previous.get()
            ),
            Self::CapacityExhausted => {
                formatter.write_str("display topology replacement allocation exhausted capacity")
            }
        }
    }
}

impl Error for DisplayTopologyReplacementError {}

impl DisplayTopologySnapshot {
    /// Builds the event payload for publishing this snapshot after `previous`.
    ///
    /// This is a low-frequency publication operation. It first counts each
    /// output class, reserves exact result capacity, then performs the stable
    /// second pass. Snapshot lookup remains O(1) by display identity.
    pub fn replacement_from(
        &self,
        previous: &Self,
    ) -> Result<DisplayTopologyReplacement, DisplayTopologyReplacementError> {
        let previous_generation = previous.generation();
        let current_generation = self.generation();
        if current_generation <= previous_generation {
            return Err(DisplayTopologyReplacementError::GenerationNotAdvanced {
                previous: previous_generation,
                current: current_generation,
            });
        }

        let (added_count, changed_count) = self.displays().fold(
            (0_usize, 0_usize),
            |(added, changed), display| match previous.get(display.id()) {
                None => (added + 1, changed),
                Some(previous_display) if previous_display != display => (added, changed + 1),
                Some(_) => (added, changed),
            },
        );
        let removed_count = previous
            .displays()
            .filter(|display| !self.contains(display.id()))
            .count();

        let mut added = reserve_ids(added_count)?;
        let mut removed = reserve_ids(removed_count)?;
        let mut changed = reserve_ids(changed_count)?;
        for display in self.displays() {
            match previous.get(display.id()) {
                None => added.push(display.id().clone()),
                Some(previous_display) if previous_display != display => {
                    changed.push(display.id().clone());
                }
                Some(_) => {}
            }
        }
        for display in previous.displays() {
            if !self.contains(display.id()) {
                removed.push(display.id().clone());
            }
        }

        Ok(DisplayTopologyReplacement::new(
            previous_generation,
            current_generation,
            added,
            removed,
            changed,
            self.primary_display_id() != previous.primary_display_id(),
        ))
    }
}

fn reserve_ids(count: usize) -> Result<Vec<DisplayId>, DisplayTopologyReplacementError> {
    let mut ids = Vec::new();
    ids.try_reserve_exact(count)
        .map_err(|_| DisplayTopologyReplacementError::CapacityExhausted)?;
    Ok(ids)
}
